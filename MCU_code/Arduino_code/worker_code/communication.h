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
void sendtoMCUs(const char* message, std::vector<byte>& MCUs,const byte cur_mcu,byte* cur_input,int& rec_count,byte& send_count,int* split_point){
  for(byte m : MCUs){
    if(m == cur_mcu){
      for(int i = 2; i <send_count + 2; i++ ){
        int pos = 0;
        for(int j = 0; j <= cur_mcu; j++){
          pos += split_point[j];
        }
        cur_input[pos] = message[i];
        split_point[cur_mcu] += 1;
        rec_count += 1;
      }
    }
    else{
      if(m == 0){
        sendUDPMessage(message,ip1,localPort);
      }
      if(m == 1){
        sendUDPMessage(message,ip2,localPort);
      }
      if(m == 2){
        sendUDPMessage(message,ip3,localPort);
      }      
    } 
  }
}
void check_and_receive(int* split_point,int& rec_count,byte* input_distribution,int input_size){
  while(udp.parsePacket()){
      // Allocate buffer to hold incoming data
      char packetBuffer[UDP_TX_PACKET_MAX_SIZE];
      // Read incoming packet into buffer
      udp.read(packetBuffer, UDP_TX_PACKET_MAX_SIZE);
      // udp.flush();
      byte from_which = packetBuffer[0];
      byte length = packetBuffer[1];
      for(int i = 2; i < length + 2; i++){
        int pos = 0;
        for(int j = 0; j <= from_which; j++){
          pos += split_point[j];
        }
        input_distribution[pos] = packetBuffer[i];
        rec_count += 1;
        split_point[from_which] += 1;
        if(rec_count == input_size) return;
      }
  }
}
#endif
