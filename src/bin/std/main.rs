mod mpsc;
mod tcp;
mod udp;

fn main() {
    mpsc::sleeping();
    mpsc::spinning();

    tcp::sleeping();
    tcp::spinning();

    udp::sleeping();
    udp::spinning();
}
