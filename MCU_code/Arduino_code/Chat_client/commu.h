#ifndef COMMUNICATION_H
#define COMMUNICATION_H

#include <NativeEthernet.h>
#include <NativeEthernetUdp.h>

const int num_mcu = 3;

// Define MAC addresses for each MCU
byte mac1[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0x01 };
byte mac2[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0x02 };
byte mac3[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0x03 };

// Define IP addresses for each MCU
IPAddress ip1(192, 168, 1, 101);
IPAddress ip2(192, 168, 1, 102);
IPAddress ip3(192, 168, 1, 103);

// Define ports for each MCU
int port1 = 10001;
int port2 = 10002;
int port3 = 10003;

EthernetClient client[num_mcu - 1];
EthernetServer server(port1);
EthernetClient client_rev[num_mcu - 1];
void server_rec(EthernetClient* clients){
  EthernetClient newClient = server.accept();
  Serial.print("rec");
  if (newClient) {
    for (byte i=0; i < num_mcu; i++) {
      if (!clients[i]) {
        Serial.print("We have a new client #");
        Serial.println(i);
        clients[i] = newClient;
        break;
      }
      delay(1000);
    }
  }
}
void setup_communication(IPAddress ip, uint8_t* mac) {
    Ethernet.setStackHeap(10 * 1024);
    Ethernet.setSocketSize(1 * 1024);
    Ethernet.setSocketNum(3);

    Ethernet.begin(mac, ip);
    while (!client[0] || !client[1]) {
      // Connect to other MCUs
      for (int i = 0; i < num_mcu - 1; i++) {
          if (memcmp(mac, mac1, sizeof(mac1)) == 0 && i == 0) {
              server_rec(client_rev);
              if(client[i].connect(ip2, port2)) Serial.println("connected to MCU2"); // Connect to MCU 2
          } else if (memcmp(mac, mac1, sizeof(mac1)) == 0 && i == 1) {
              server_rec(client_rev);
              if(client[i].connect(ip3, port3)) Serial.println("connected to MCU3"); // Connect to MCU 3
          } else if (memcmp(mac, mac2, sizeof(mac2)) == 0 && i == 0) {
              server_rec(client_rev);
              if(client[i].connect(ip1, port1)) Serial.println("connected to MCU1"); // Connect to MCU 1
          } else if (memcmp(mac, mac2, sizeof(mac2)) == 0 && i == 1) {
              server_rec(client_rev);
              if(client[i].connect(ip3, port3)) Serial.println("connected to MCU3"); // Connect to MCU 3
          } else if (memcmp(mac, mac3, sizeof(mac3)) == 0 && i == 0) {
              server_rec(client_rev);
              if(client[i].connect(ip1, port1)) Serial.println("connected to MCU1"); // Connect to MCU 1
          } else if (memcmp(mac, mac3, sizeof(mac3)) == 0 && i == 1) {
              server_rec(client_rev);
              if(client[i].connect(ip2, port2)) Serial.println("connected to MCU2"); // Connect to MCU 2
          }
      }
      delay(2000);
    }
}
#endif
