#include <NativeEthernet.h>
#include <NativeEthernetUdp.h>

// Define MAC addresses for each MCU
byte mac1[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0x01 };
byte mac2[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0x02 };
byte mac3[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0x03 };

// Define IP addresses and ports for each MCU
IPAddress ip1(192, 168, 1, 101);
IPAddress ip2(192, 168, 1, 102);
IPAddress ip3(192, 168, 1, 103);

unsigned int localPort = 8888;  // Local port to listen on for UDP packets
EthernetUDP udp;
int count = 0;
void setup() {
  // Initialize Ethernet and UDP
  Ethernet.setStackHeap(60 * 1024);
  Ethernet.setSocketSize(100 * 1024);
  Ethernet.setSocketNum(3);
  Ethernet.begin(mac2, ip2); // Change to mac2 and ip2 for MCU2, mac3 and ip3 for MCU3
  udp.begin(localPort);
  // Serial.println("!!");
  // char packet[20] = "2 to 1";
}

void loop() {
  // if(count == 0){
  //   sendUDPMessage("1 to 2", ip2, localPort);    
  // }
  // Check if data is available to be read
  int packetSize = udp.parsePacket();
  // char c[250];
  // c[0] = 'a' + count;
  // sendUDPMessage(c, ip2, localPort);
  count += 1;
  if (packetSize) {
    // Allocate buffer to hold incoming data
    char packetBuffer[UDP_TX_PACKET_MAX_SIZE];
    // Read incoming packet into buffer
    udp.read(packetBuffer, UDP_TX_PACKET_MAX_SIZE);
    udp.flush();
    // Print received packet
    Serial.print("Received packet: ");
    Serial.println(packetBuffer);
  }
  // char packet[20] = "2 to 1";
  // byte to_send[20] = {48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67};
  // for(int i = 0; i < 20; i++){
  //   packet[i] = (char)to_send[i];
  // }
  // sendUDPMessage("2 to 1", ip1, localPort);
  // sendUDPMessage("2 to 3", ip3, localPort);
  delay(100);  // Delay for 1 second
}

// Function to send UDP message
void sendUDPMessage(const char* message, IPAddress destinationIP, unsigned int destinationPort) {
  udp.beginPacket(destinationIP, destinationPort);
  udp.write(message);
  udp.endPacket();
}
