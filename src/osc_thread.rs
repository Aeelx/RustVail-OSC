use crate::ThreadData;
use nannou_osc::Type::Float;
use nannou_osc::{Message, Sender};
use std::sync::mpsc;



pub fn thread(rx: mpsc::Receiver<ThreadData>) {
    //hang thread until we get the first message from the first render
    let mut data = rx.recv().unwrap();

    let mut osc_tx = Sender::bind()
    .expect("Couldn't bind to default socket")
    .connect(&data.ip_address)
    .expect("Couldn't connect to socket at address");

    //loop with timeout so we can send a value at least once a second even if the other thread stops/slows down rendering
    loop {
        match rx.recv_timeout(std::time::Duration::from_millis(300)) {
            Ok(value) => data = value,
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(err) => panic!("Unexpected error: {:?}", err),
        };

        //if ip address changed, reconnect
        if data.ip_address != osc_tx.remote_addr().to_string() {
            osc_tx = Sender::bind()
            .expect("Couldn't bind to default socket")
            .connect(&data.ip_address)
            .expect("Couldn't connect to socket at address");
        }

        osc_loop(&osc_tx, &data)
    }
}

fn osc_loop(osc_tx: &Sender<nannou_osc::Connected>, thread_data: &ThreadData) {
    //check if we should send anything
    if !thread_data.enabled {
        return;
    }

    //create messages to send
    let left_foot = Message {
        addr: "/tracking/trackers/1/position".to_string(),
        args: [Float(-0.1), Float(0.0 + thread_data.height), Float(0.0)].to_vec(),
    };
    let right_foot = Message {
        addr: "/tracking/trackers/2/position".to_string(),
        args: [Float(0.1), Float(0.0 + thread_data.height), Float(0.0)].to_vec(),
    };
    let hip = Message {
        addr: "/tracking/trackers/3/position".to_string(),
        args: [Float(0.0), Float(0.9 + thread_data.height), Float(0.0)].to_vec(),
    };
    let head_position = Message {
        addr: "/tracking/trackers/head/position".to_string(),
        args: [Float(0.0), Float(1.75 + thread_data.height), Float(0.0)].to_vec(),
    };
    let head_rotation = Message {
        addr: "/tracking/trackers/head/rotation".to_string(),
        args: [Float(0.0), Float(0.0), Float(0.0)].to_vec(),
    };

    //send only whats enabled
    if thread_data.hip_enabled {
        osc_tx.send(hip).unwrap();
    }
    if thread_data.left_foot_enabled {
        osc_tx.send(left_foot).unwrap();
    }
    if thread_data.right_foot_enabled {
        osc_tx.send(right_foot).unwrap();
    }
    if thread_data.locked_to_headset {
        osc_tx.send(head_position).unwrap();
        osc_tx.send(head_rotation).unwrap();
    }
}
