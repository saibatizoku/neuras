extern crate futures;
extern crate neuras;
extern crate tokio_core;
extern crate tokio_signal;
extern crate zmq;

use std::thread;
use std::time::Duration;
use std::io::Error as IoError;

use futures::{stream, FlattenStream, Future, Stream};
use futures::stream::Take;
use futures::future::{err, ok};
use neuras::actor::Actorling;
use neuras::actor::errors::*;
use tokio_core::reactor::{Core, Handle};

const POLL_TIMEOUT: i64 = 100;

fn run_pipe(actor: &Actorling) -> Result<thread::JoinHandle<Result<()>>> {
    let addr = actor.pipe_address();
    let context = actor.context();
    let pipe_thread = actor
        .run_thread(move || {
            println!("running pipe thread");
            let pipe = context.socket(zmq::PAIR)?;
            let _ = pipe.bind(&addr)?;
            let controller = context.socket(zmq::PUB)?;
            let _ = controller.bind("inproc://controller")?;
            let mut msg = zmq::Message::new();
            let mut items = [pipe.as_poll_item(zmq::POLLIN | zmq::POLLOUT)];
            loop {
                let _ = zmq::poll(&mut items, POLL_TIMEOUT)?;
                if items[0].is_readable() {
                    eprintln!("pipe is readable");
                    let _ = pipe.recv(&mut msg, 0)?;
                    match msg.as_str() {
                        Some(a) if a == "STOP" => {
                            println!("pipe sending STOP signal");
                            let _ = controller.send("STOP", 0)?;
                            println!("pipe is stopping");
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
            println!("exiting pipe thread");
            Ok(())
        })
        .unwrap();
    Ok(pipe_thread)
}

fn run_public(actor: &Actorling) -> Result<thread::JoinHandle<Result<()>>> {
    let addr = actor.address();
    let context = actor.context();
    let public_thread = actor
        .run_thread(move || {
            println!("running public thread");
            let public = context.socket(zmq::REP)?;
            let _ = public.bind(&addr)?;
            let controller = context.socket(zmq::SUB)?;
            let _ = controller.connect("inproc://controller")?;
            let _ = controller.set_subscribe(b"")?;
            let mut msg = zmq::Message::new();
            let mut items = [
                public.as_poll_item(zmq::POLLIN | zmq::POLLOUT),
                controller.as_poll_item(zmq::POLLIN),
            ];
            loop {
                let _ = zmq::poll(&mut items, POLL_TIMEOUT)?;
                if items[1].is_readable() {
                    let _ = controller.recv(&mut msg, 0)?;
                    match msg.as_str() {
                        Some(a) if a == "STOP" => {
                            println!("public is stopping");
                            let _ = public.disconnect(&addr)?;
                            let _ = controller.set_unsubscribe(b"")?;
                            let _ = controller.disconnect("inproc://controller")?;
                            break;
                        }
                        _ => {}
                    }
                } else {
                    eprintln!("public controller not readable");
                }

                if items[0].is_readable() {
                    let _ = public.recv(&mut msg, 0)?;
                    eprintln!("public is readable");
                    match msg.as_str() {
                        Some(a) => {
                            println!("ECHO {}", a);
                            let _ = public.send(a, 0)?;
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
            println!("exiting public thread");
            Ok(())
        })
        .unwrap();
    Ok(public_thread)
}

fn run_playful(actor: &Actorling) -> Result<thread::JoinHandle<Result<()>>> {
    let addr = actor.address();
    let context = actor.context();
    let public_thread = actor
        .run_thread(move || {
            println!("running play thread");
            let mut core = Core::new().unwrap();
            let public = context.socket(zmq::REQ).unwrap();
            let _ = public.connect(&addr).unwrap();
            let controller = context.socket(zmq::SUB).unwrap();
            let _ = controller.connect("inproc://controller").unwrap();
            let _ = controller.set_subscribe(b"").unwrap();
            let c = &controller;

            let mut msg = zmq::Message::new();
            let mut stream = stream::unfold(c, |controller| {
                let _ = controller.recv(&mut msg, 0).unwrap();
                let fut = match msg.as_str() {
                    Some(m) if m == "STOP" => {
                        println!("stopping play");
                        return None;
                    }
                    Some(m) => ok::<_, ()>((m.to_string(), controller)),
                    None => err::<_, ()>(()),
                };
                Some(fut)
            });
            let _future = match stream.poll() {
                Ok(futures::Async::Ready(Some(a))) => ok::<String, ()>(a),
                Ok(futures::Async::Ready(None)) => {
                    println!("finished play");
                    let _ = controller.set_unsubscribe(b"").unwrap();
                    let _ = controller.disconnect("inproc://controller").unwrap();
                    println!("play thread interrupted");
                    return Ok(());
                }
                _ => err::<String, ()>(()),
            };
            let fut = _future
                .and_then(|msg| {
                    println!("msg: {:?}", msg);
                    Ok(&controller)
                })
                .and_then(|_| {
                    println!("using it again!");
                    Ok(())
                });
            let _ = core.run(fut).unwrap();
            println!("exiting play thread");
            Ok(())
        })
        .unwrap();

    Ok(public_thread)
}

type FutureStream =
    Future<Item = Box<Stream<Error = IoError, Item = ()> + Send>, Error = IoError> + Send;

type SignalInterruption = Take<FlattenStream<Box<FutureStream>>>;

fn catch_sigint(handle: &Handle) -> Result<SignalInterruption> {
    let ctrl_c = tokio_signal::ctrl_c(&handle).flatten_stream().take(1);
    Ok(ctrl_c)
}

fn main() {
    let mut core = Core::new().unwrap();
    let actor = Actorling::new("tcp://127.0.0.1:8889").unwrap();

    // spawn the pipe thread.
    let pipe_thread = run_pipe(&actor).unwrap();
    // spawn the public actorling thread.
    let public_thread = run_public(&actor).unwrap();
    // spawn the public actorling thread.
    let play_thread = run_playful(&actor).unwrap();

    let threads = vec![pipe_thread, public_thread, play_thread];

    // Run a tokio reactor to catch incoming signals from CTRL-C.
    // Returns a future that exits with the first signal it receives.
    let proc_handle = catch_sigint(&core.handle()).unwrap();
    let proc_interrupt = proc_handle.for_each(|_| {
        // create a socket to connect to the pipe thread
        let sender = actor.context().socket(zmq::PAIR)?;
        let _ = sender.connect(&actor.pipe_address())?;

        println!();
        println!("SIGINT received: Actorling pipe-thread will be interrupted!");

        // Send the `STOP` message to the pipe thread.
        let _ = sender.send("STOP", 0)?;
        Ok(())
    });

    // Run the event-loop checking for CTRL-C interrupts.
    //
    // This will loop and block the main thread until the user presses
    // CTRL-C or a SIGINT is sent via the usual unix-like fashion.
    let _ = core.run(proc_interrupt).unwrap();

    // Wait for the child threads to join.
    for t in threads {
        let _ = t.join().unwrap();
    }
    // Exit cleanly.
    ::std::process::exit(0)
}
