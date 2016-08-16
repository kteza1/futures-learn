extern crate futures;
extern crate futures_io;
extern crate futures_mio;
extern crate futures_tls;
extern crate openssl;
extern crate mqtt;

use std::net::ToSocketAddrs;
use std::thread;
use std::time::Duration;

use futures::Future;
use futures_mio::Loop;
use futures_mio::TcpStream;
use futures_tls::ClientContext;
use futures_tls::TlsStream;
use futures_tls::backend::openssl::ClientContextExt;
use openssl::x509::X509FileType;

use mqtt::{Encodable, Decodable, QualityOfService, TopicFilter};
use mqtt::packet::*;
use mqtt::control::variable_header::{ConnectReturnCode, PacketIdentifier};
use mqtt::topic_name::TopicName;

struct MqttClient {
    stream: TlsStream<TcpStream>,
}

fn main() {
    let mut connection_lp = Loop::new().unwrap();
    let addr =
        "veh-test-mqtt-broker.atherengineering.in:8883".to_socket_addrs().unwrap().next().unwrap();

    let stream = connection_lp.handle().tcp_connect(&addr);

    let tls_handshake = stream.and_then(|stream| {
        // client.stream = Some(socket);
        let mut cx = ClientContext::new().unwrap();
        {
            let backend_cx = cx.ssl_context_mut();
            let _ = backend_cx.set_CA_file("/home/raviteja/Desktop/mqtt_gcloud_certs/ca.crt");
            let _ = backend_cx.set_certificate_file("/home/raviteja/Desktop/mqtt_gcloud_certs/client_mqtt_veh_test.crt", X509FileType::PEM);
            let _ = backend_cx.set_private_key_file("/home/raviteja/Desktop/mqtt_gcloud_certs/client_mqtt_veh_test.key", X509FileType::PEM);
        }
        cx.handshake("veh-test-mqtt-broker.atherengineering.in", stream)
    });

    // Mqtt connection created
    let mqtt_connect = tls_handshake.and_then(|stream| {
        let packet = _generate_connect_packet().unwrap();
        futures_io::write_all(stream, packet)
    });

    // Mqtt Connection created
    let (stream, _) = connection_lp.run(mqtt_connect).unwrap();

    // Send Ping Requests periodically over existing stream
    let mut timeout_lp = Loop::new().unwrap();
    let timeout = timeout_lp.handle().timeout(Duration::new(4, 0)).map(|t| {
        let packet = _generate_pingreq_packet().unwrap();
        futures_io::write_all(stream, packet)
    });
    timeout_lp.run(timeout).unwrap();
    thread::sleep(Duration::new(20, 0));
}

fn _generate_connect_packet() -> Result<Vec<u8>, i32> {
    let mut connect_packet = ConnectPacket::new("MQTT".to_owned(), "test-id".to_string());

    connect_packet.set_clean_session(true);
    connect_packet.set_keep_alive(5);

    let mut buf = Vec::new();

    connect_packet.encode(&mut buf).expect("Encode fail");
    Ok(buf)
}

fn _generate_pingreq_packet() -> Result<Vec<u8>, i32> {
    let pingreq_packet = PingreqPacket::new();
    let mut buf = Vec::new();

    pingreq_packet.encode(&mut buf).expect("Encode fail");
    Ok(buf)
}
