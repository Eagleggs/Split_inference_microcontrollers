#include <SPI.h>
#include <NativeEthernet.h>
byte mac[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xEB }; // MAC address
IPAddress serverIP(192, 168, 1, 177); // IP address of the server
IPAddress ip(192, 168, 1, 175);
IPAddress myDns(192, 168, 1, 1);
IPAddress gateway(192, 168, 1, 1);
IPAddress subnet(255, 255, 0, 0);
EthernetClient client; // Create a client object

void setup() {
  Ethernet.begin(mac, ip, myDns, gateway, subnet);
  if (client.connect(serverIP, 23)) { // Connect to the server
  } 
}

void loop() {
  char to_send[20] ="1";
  if (client.connected()) { // If connected to server
    client.println(to_send); // Send data to server
    delay(1000); // Wait for a second
  }
}
