#ifndef COMMUNICATION_H
#define COMMUNICATION_H

#include <NativeEthernet.h>
#include <NativeEthernetUdp.h>

const int num_mcu = 3;

// Define MAC addresses for each MCU
byte mac1[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0x01 };
byte mac2[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0x02 };
byte mac3[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0x03 };

// Define IP addresses and ports for each MCU
IPAddress ip1(192, 168, 1, 101);
IPAddress ip2(192, 168, 1, 102);
IPAddress ip3(192, 168, 1, 103);

unsigned int port1 = 10001; 
unsigned int port2 = 10002; 

unsigned int localPort = 8888;  // Local port to listen on for UDP packets
EthernetUDP udp;

void setup_communication(IPAddress ip, uint8_t* mac) {
  Ethernet.setStackHeap(20 * 1024);
  Ethernet.setSocketSize(1 * 1024);
  Ethernet.setSocketNum(3);
  Ethernet.begin(mac, ip); // Change to mac2 and ip2 for MCU2, mac3 and ip3 for MCU3
  udp.begin(localPort);
}
void sendUDPMessage(const char* message, IPAddress destinationIP, unsigned int destinationPort) {
  udp.beginPacket(destinationIP, destinationPort);
  udp.write(message);
  udp.endPacket();
}
#endif
