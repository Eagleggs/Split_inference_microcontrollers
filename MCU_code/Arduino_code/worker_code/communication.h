#ifndef COMMUNICATION_H
#define COMMUNICATION_H
#define UDP_TX_PACKET_MAX_SIZE 250

#include <QNEthernet.h>
using namespace qindesign::network;

const int num_mcu = 3;

// Define MAC addresses for each MCU
IPAddress subnetMask{255, 255, 255, 0};
IPAddress gateway{192, 168, 0, 1};

// Define IP addresses and ports for each MCU
IPAddress ip1(192, 168, 1, 101);
IPAddress ip2(192, 168, 1, 102);
IPAddress ip3(192, 168, 1, 103);


unsigned int localPort = 8888;  // Local port to listen on for UDP packets
EthernetUDP udp;

void setup_communication(IPAddress ip) {
  if(!Ethernet.begin(ip,subnetMask,gateway)) Serial.println("start ethernet failed"); // Change to mac2 and ip2 for MCU2, mac3 and ip3 for MCU3
  if(udp.begin(localPort) == 0) Serial.println("start udp failed");
}
void sendUDPMessage(const char* message, IPAddress destinationIP, unsigned int destinationPort) {
  if(!udp.send(destinationIP,destinationPort,message,UDP_TX_PACKET_MAX_SIZE)){
    Serial.println("send failed");
  }
}
void sendtoMCUs(const char* message, std::vector<byte>& MCUs,const byte cur_mcu,byte* cur_input,int& rec_count,byte& send_count,int* split_point,int input_size){
  for(byte m : MCUs){
    if(m == cur_mcu){
      for(int i = 2; i <send_count + 2; i++ ){
        int pos = 0;
        for(int j = 0; j <= cur_mcu; j++){
          pos += split_point[j];
        }
        // if(pos < rec_count){
        //   for(int j = input_size - 1; j > pos;j-- ){
        //       cur_input[j] = cur_input[j - 1];
        //   }          
        // }
        cur_input[pos] = message[i];
        split_point[cur_mcu] += 1;
        rec_count += 1;
        // Serial.println(rec_count);
      }
    }
    else{
      if(m == 0){
        sendUDPMessage(message,ip1,localPort);
        Serial.println("send to 1");
      }
      if(m == 1){
        byte c = message[0];
        sendUDPMessage(message,ip2,localPort);
        Serial.println("send to 2");
      }
      if(m == 2){
        sendUDPMessage(message,ip3,localPort);
        Serial.println("send to 3");
      }      
    } 
  }
}
void check_and_receive(int* split_point,int& rec_count,byte* input_distribution,int input_size){
  while(udp.parsePacket() > 0){
      Serial.println("received packet");
      // Allocate buffer to hold incoming data
      char packetBuffer[UDP_TX_PACKET_MAX_SIZE];
      // Read incoming packet into buffer
      udp.read(packetBuffer, UDP_TX_PACKET_MAX_SIZE);
      byte from_which = packetBuffer[0];
      byte length = packetBuffer[1];
      Serial.print("receiving from");
      Serial.print(from_which);
      Serial.println(rec_count);
      for(int i = 2; i < length + 2; i++){
        int pos = 0;
        for(int j = 0; j <= from_which; j++){
          pos += split_point[j];
        }
        // if(pos < rec_count){
        //   for(int j = input_size - 1; j > pos;j-- ){
        //       input_distribution[j] = input_distribution[j - 1];
        //   }          
        // }
        input_distribution[pos] = packetBuffer[i];
        rec_count += 1;
        split_point[from_which] += 1;
        if(rec_count == input_size) return;
      }
  }
}
#endif
