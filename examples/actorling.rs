#![feature(unboxed_closures)]
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate neuras;
extern crate tokio_core;
extern crate tokio_signal;
extern crate zmq;

use std::thread;
use std::time::Duration;
use std::io::Error as IoError;

use futures::{FlattenStream, Future, Stream};
use futures::future::{err, ok};
use futures::stream;
use futures::stream::Take;
use neuras::actor::Actorling;
use neuras::actor::errors::*;
use tokio_core::reactor::{Core, Handle};

const POLL_TIMEOUT: i64 = 100;

fn run_pipe_thread(actor: &Actorling) -> Result<thread::JoinHandle<Result<()>>> {
    let addr = actor.address();
    let context = actor.context();
    let pipe_thread = actor
        .run_thread("pipe", move || {
            println!("enter: pipe thread");
            let pipe = context.socket(zmq::PAIR)?;
            pipe.bind(&addr)?;
            let controller = context.socket(zmq::PUB)?;
            controller.bind("inproc://controller")?;
            let mut msg = zmq::Message::new();
            let mut items = [pipe.as_poll_item(zmq::POLLIN | zmq::POLLOUT)];
            loop {
                zmq::poll(&mut items, POLL_TIMEOUT)?;
                if items[0].is_readable() {
                    eprintln!("pipe is readable");
                    pipe.recv(&mut msg, 0)?;
                    match msg.as_str() {
                        Some(a) if a == "STOP" => {
                            println!("broadcast: STOP");
                            controller.send("STOP", 0)?;
                            eprintln!("stop: pipe");
                            break;
                        }
                        _ => {}
                    }
                } else {
                    eprintln!("pipe not readable");
                }
                if items[0].is_writable() {
                    eprintln!("pipe is writable");
                } else {
                    eprintln!("pipe not writable");
                }
                thread::sleep(Duration::from_millis(POLL_TIMEOUT as u64));
            }
            println!("exit: pipe thread");
            Ok(())
        })
        .unwrap();
    Ok(pipe_thread)
}

fn run_public(actor: &Actorling) -> Result<thread::JoinHandle<Result<()>>> {
    let addr = actor.address();
    let context = actor.context();
    let public_thread = actor
        .run_thread("public", move || {
            println!("enter: public thread");
            let public = context.socket(zmq::REP)?;
            public.bind(&addr)?;
            let controller = context.socket(zmq::SUB)?;
            controller.connect("inproc://controller")?;
            controller.set_subscribe(b"")?;
            let mut msg = zmq::Message::new();
            let mut items = [
                public.as_poll_item(zmq::POLLIN | zmq::POLLOUT),
                controller.as_poll_item(zmq::POLLIN),
            ];
            loop {
                zmq::poll(&mut items, POLL_TIMEOUT)?;
                if items[1].is_readable() {
                    controller.recv(&mut msg, 0)?;
                    match msg.as_str() {
                        Some(a) if a == "STOP" => {
                            eprintln!("stop: public");
                            public.disconnect(&addr)?;
                            controller.set_unsubscribe(b"")?;
                            controller.disconnect("inproc://controller")?;
                            break;
                        }
                        _ => {}
                    }
                } else {
                    eprintln!("public controller not readable");
                }

                if items[0].is_readable() {
                    public.recv(&mut msg, 0)?;
                    eprintln!("public is readable");
                    match msg.as_str() {
                        Some(a) => {
                            println!("ECHO {}", a);
                            public.send(a, 0)?;
                        }
                        _ => {}
                    }
                } else {
                    eprintln!("public not readable");
                }
                if items[0].is_writable() {
                    eprintln!("public is writable");
                } else {
                    eprintln!("public not writable");
                }
                thread::sleep(Duration::from_millis(POLL_TIMEOUT as u64));
            }
            println!("exit: public thread");
            Ok(())
        })
        .unwrap();
    Ok(public_thread)
}

fn control_pipe_stream(context: zmq::Context) -> Result<()> {
    let mut core = Core::new().unwrap();
    let controller = context.socket(zmq::SUB).unwrap();
    controller.connect("inproc://controller").unwrap();
    controller.set_subscribe(b"").unwrap();
    let mut msg = zmq::Message::new();
    let control_pipe = stream::unfold(controller, |controller| {
        controller.recv(&mut msg, 0).unwrap();
        let fut = match msg.as_str() {
            Some(m) if m == "STOP" => {
                eprintln!("stop: play");
                controller.set_unsubscribe(b"").unwrap();
                controller.disconnect("inproc://controller").unwrap();
                return None;
            }
            Some(m) => ok::<(String, zmq::Socket), ()>((m.to_string(), controller)),
            None => err::<(String, zmq::Socket), ()>(()),
        };
        Some(fut)
    }).for_each(|msg| {
        println!("msg: {:?}", msg);
        Ok(())
    });
    core.run(control_pipe).unwrap();
    Ok(())
}

fn run_playful(actor: &Actorling) -> Result<thread::JoinHandle<Result<()>>> {
    let addr = actor.address();
    let context = actor.context();
    let public_thread = actor
        .run_thread("play", move || {
            println!("enter: play thread");
            let public = context.socket(zmq::REQ).unwrap();
            public.connect(&addr).unwrap();

            control_pipe_stream(context).unwrap();

            public.disconnect(&addr).unwrap();
            println!("exit: play thread");
            Ok(())
        })
        .unwrap();

    Ok(public_thread)
}

type FutureStream =
    Future<Item = Box<Stream<Error = IoError, Item = ()> + Send>, Error = IoError> + Send;

type SignalInterruption = Take<FlattenStream<Box<FutureStream>>>;

fn catch_sigint(handle: &Handle) -> SignalInterruption {
    tokio_signal::ctrl_c(handle).flatten_stream().take(1)
}

fn main() {
    let actor = Actorling::new("tcp://127.0.0.1:8889").unwrap();

    // spawn the pipe thread.
    let pipe_thread = run_pipe_thread(&actor).unwrap();
    // spawn the public actorling thread.
    let public_thread = run_public(&actor).unwrap();
    // spawn the public actorling thread.
    let play_thread = run_playful(&actor).unwrap();

    let threads = vec![pipe_thread, public_thread, play_thread];

    println!("starting main process");
    // Run a tokio reactor to catch incoming signals from CTRL-C.
    // Returns a future that exits with the first signal it receives.
    let mut core = Core::new().unwrap();
    let proc_handle = catch_sigint(&core.handle())
        .and_then(|_| {
            // create a socket to connect to the pipe thread
            let sender = actor.context().socket(zmq::PAIR)?;
            sender.connect(&actor.address())?;
            Ok(sender)
        })
        .and_then(|sender| {
            println!();
            println!("SIGINT received: Actorling pipe-thread will be interrupted!");
            Ok(sender)
        })
        .for_each(|sender| {
            // Send the `STOP` message to the pipe thread.
            eprintln!("control: STOP");
            sender.send("STOP", 0)?;
            Ok(())
        });

    // Run the event-loop checking for CTRL-C interrupts.
    //
    // This will loop and block the main thread until the user presses
    // CTRL-C or a SIGINT is sent via the usual unix-like fashion.
    core.run(proc_handle).unwrap();

    // Exit cleanly by joining children threads into the main thread.
    // Returns the `exitCode` for this process.
    let exit_code = match cleanup_n_exit(threads) {
        Ok(_) => 0,
        Err(_) => 1,
    };
    println!("exiting main process");
    ::std::process::exit(exit_code)
}

fn cleanup_n_exit(threads: Vec<thread::JoinHandle<Result<()>>>) -> Result<()> {
    // Wait for the child threads to join.
    for t in threads {
        let name = match t.thread().name() {
            Some(n) => n.to_string(),
            None => format!("{:?}", t.thread().id()),
        };
        match t.join() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("thread not joined {:?}", e);
                bail!("thread not joined");
            }
        }
        println!("joined {:?} thread with parent", name);
    }
    Ok(())
}
