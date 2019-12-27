/// tps://adventofcode.com/2019/day/23

mod intcode;
use intcode::Error;
use intcode::Error::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use futures::prelude::*;
use futures::channel::mpsc::{channel,Sender,Receiver};
use futures::executor::block_on;
use futures::join;
use futures::future::join_all; // https://rust-lang.github.io/async-book/07_workarounds/05_recursion.html

fn main() -> Result<(),Error> {
    const PROG_MEM_SIZE: usize = 3000;
    let filename = "input.txt";
    let fd = File::open(filename).expect(&format!("Failure opening {}", filename));
    let buf = BufReader::new(fd);
    let mut prog_orig = Vec::new();
    buf.lines().for_each(|line| {
        line.unwrap().split(',').for_each(|numstr| {
            let num = numstr.parse::<isize>().unwrap();
            prog_orig.push(num);
        });
    });
    // Add some empty space for code growth
    if prog_orig.len() < PROG_MEM_SIZE {
        let mut extra_space = vec![0; PROG_MEM_SIZE - prog_orig.len()];
        prog_orig.append(&mut extra_space);
    };
    let (part1, part2) = match block_on(boot_50_intcode_machines(prog_orig)) {
        Ok(result) => result,
        Err(e) => return Err(e),
    };
    println!("");
    println!("Part 1: TBD {}", part1 );
    println!("Part 2: TBD {}", part2 );
    Ok(())
}
async fn boot_50_intcode_machines(prog: Vec<isize>) -> Result<(isize,isize),Error> {
    const BUFFER_SIZE: usize = 10;
    const COMPUTER_COUNT: usize = 50;
    let mut network_rx = Vec::new();
    let mut network_tx = Vec::new();
    let mut computers = Vec::new();
    for _comp in 0..COMPUTER_COUNT {
        let (net_tx, comp_rx) = channel::<isize>(BUFFER_SIZE);
        let (comp_tx, net_rx) = channel::<(isize,isize,isize)>(BUFFER_SIZE);
        network_rx.push(net_rx);
        network_tx.push(net_tx);
        let computer = intcode::Intcode::new(prog.clone(), comp_rx, comp_tx);
        computers.push(computer);
    }
    initialize_addresses(&mut network_tx).await?;
    let net = manage_network(network_rx, network_tx);
    let run_all_computers = join_all((&mut computers).into_iter().map(|c| {c.run()}));
    let (_computer_repsonses, net_response) = join!(run_all_computers, net);
    net_response
}
async fn manage_network(mut rx: Vec<Receiver<(isize,isize,isize)>>, mut tx: Vec<Sender<isize>>) -> Result<(isize,isize),Error> {
    let rx_data = (&mut rx).into_iter().map(|rx|{rx.next()});
    let mut compute_farm = futures::future::select_all(rx_data);
    loop {
        let (some_tuple, from_addr,remaining) = compute_farm.await;
        match some_tuple {
            None => panic!("Received None as data from compute farm!"),
            Some((to_addr, x, y)) => {
                if to_addr == 255 {
                    return Ok((y,0));
                } else {
                    println!("From: {} To: {} -- ({},{}) ", from_addr, to_addr, x, y);
                    send_x_y(&mut tx[to_addr as usize], x, y).await?;
                }
            },
        }
        // remaining.push(rx[from_addr].next());
        println!("remaining.len(): {}", remaining.len());
        compute_farm = futures::future::select_all(remaining);
    }
}
async fn initialize_addresses(tx_list: &mut Vec<Sender<isize>>) -> Result<(),Error> {
    let mut addr = 0isize;
    for tx in tx_list {
        if let Err(_) = tx.send(addr).await {
            // println!("Intcode Reporting WRITE error");
            return Err(ComputerComms{msg:"Problem sending initial address. Has receiver been dropped?".to_string()});
        }
        if let Err(_) = tx.send(-1).await {
            // println!("Intcode Reporting WRITE error");
            return Err(ComputerComms{msg:"Problem sending initial address. Has receiver been dropped?".to_string()});
        }
        if let Err(_) = tx.send(-1).await {
            // println!("Intcode Reporting WRITE error");
            return Err(ComputerComms{msg:"Problem sending initial address. Has receiver been dropped?".to_string()});
        }
        addr += 1;
    }
    Ok(())
}
async fn send_x_y(tx: &mut Sender<isize>, x:isize, y:isize) -> Result<(),Error> {
    if let Err(_) = tx.send(x).await {
        // println!("Intcode Reporting WRITE error");
        return Err(Error::ComputerComms{msg:"Problem sending x data. Has receiver been dropped?".to_string()});
    }
    if let Err(_) = tx.send(y).await {
        // println!("Intcode Reporting WRITE error");
        return Err(Error::ComputerComms{msg:"Problem sending y data. Has receiver been dropped?".to_string()});
    }
    if let Err(_) = tx.send(-1).await {
        // println!("Intcode Reporting WRITE error");
        return Err(ComputerComms{msg:"Problem sending initial address. Has receiver been dropped?".to_string()});
    }
    if let Err(_) = tx.send(-1).await {
        // println!("Intcode Reporting WRITE error");
        return Err(ComputerComms{msg:"Problem sending initial address. Has receiver been dropped?".to_string()});
    }
Ok(())
}