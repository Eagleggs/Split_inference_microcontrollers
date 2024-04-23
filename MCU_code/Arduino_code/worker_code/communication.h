//200: permission granted 199: ask for permission 198:send complete 197:ack
#ifndef COMMUNICATION_H
#define COMMUNICATION_H

#include <SPI.h>
#include <NativeEthernet.h>
#define MESSAGE_SIZE 250
const int num_mcu = 3;
const byte mcu_id = 1;
// Define IP addresses and ports for each MCU
IPAddress server(169,254,71,125);
IPAddress ip1(169,254,71,124);
IPAddress ip2(169,254,71,123);
IPAddress ip3(169,254,71,122);
byte mac1[] = {
  0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xEB
};
byte mac2[] = {
  0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xEC
};
byte mac3[] = {
  0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xED
};

unsigned int serverport = 8080;  // Local port to listen on for UDP packets
EthernetClient client;
bool permission_flag = false;
void setup_communication(IPAddress ip,const byte* mac) {
  Ethernet.setStackHeap(10 * 1024);
  Ethernet.setSocketSize(1 * 1024);
  Ethernet.setSocketNum(1);
  Ethernet.begin(mac,ip);// Change to mac2 and ip2 for MCU2, mac3 and ip3 for MCU3
  Serial.println("connecting...");
  while (!client.connect(server, serverport)) {} //connect to server
  while (!client.available()) {} //read a byte from server to indicate communication established
  char c = client.read();
  Serial.println("connection established!");
}
// void sendUDPMessage(const char* message, IPAddress destinationIP, unsigned int destinationPort) {
//   if(!udp.send(destinationIP,destinationPort,message,MESSAGE_SIZE)){
//     Serial.println("send failed");
//   }
// }
bool wait_for_ack(){
  while(!client.available()){};
  char message[MESSAGE_SIZE];
  client.readBytes(message,MESSAGE_SIZE);
  if(message[1] != 197){
    return true;
  }
  // Serial.println("ack received!");
  return false;
}
bool send_message_to_coordinator(const char* message){
  // Serial.print("sending message");
  // byte temp = message[1];
  // Serial.println(temp);
  if(client.write(message,MESSAGE_SIZE) == 0) return false;
  if(wait_for_ack()){
    Serial.println("ack message wrong, stop executing...");
    while(1){};
  };
  return true;  
}

void send_ack(){
  Serial.println("sending ack");
  char message[MESSAGE_SIZE];
  message[0] = mcu_id;
  message[1] = 197;
  client.write(message,MESSAGE_SIZE);
}

void sendtoMCUs(char* message, std::vector<byte>& MCUs,const byte cur_mcu,byte* cur_input,int& rec_count,byte& send_count){
  for(byte m : MCUs){
    if(m == cur_mcu){
      for(int i = 2; i <send_count + 2; i++ ){
        cur_input[rec_count] = message[i];
        rec_count += 1;
      }
    }
    else{
      if(m == 0){
        message[1] = 0;
        Serial.print("send to 0，message size: ");
        Serial.println((int)message[2]);      }
      if(m == 1){
        message[1] = 1;
        Serial.print("send to 1，message size: ");
        Serial.println((int)message[2]);
      }
      if(m == 2){
        message[1] = 2;
        Serial.print("send to 2，message size: ");
        Serial.println((int)message[2]);
      }
      send_message_to_coordinator(message);
    } 
  }
}
void check_and_receive(int& rec_count,byte* input_distribution){
  int count = 0;
  if(client.available()){  
    byte buffer[MESSAGE_SIZE];
    client.readBytes(buffer,MESSAGE_SIZE);
    if(buffer[1] == 200){ 
      Serial.println((byte)buffer[1]);
      permission_flag = true;
    }else{
      byte from_which = buffer[0];
      byte to_which = buffer[1]; 
      byte length = buffer[2];
      if(to_which != mcu_id){
        Serial.println("received wrong message!! Stop executing...");
        Serial.println((byte) to_which);
        while(1){};
      }
      // if(to_which != mcu_id) {
      //   Serial.print("received wrong message!This is ");
      //   Serial.print(mcu_id);
      //   Serial.print("but to ");
      //   Serial.print(to_which);

      // }else{
      //   Serial.print("received message from mcu: ");
      //   Serial.print(from_which);
      //   Serial.print("len: ");
      //   Serial.println(length);
      // }
      for(int i = 0; i < length; i++){
        input_distribution[rec_count] = buffer[i + 3];
        rec_count += 1;
      } 
    }
    send_ack();      
  }
}
void wait_for_permission(int& rec_count,byte* input_distribution){
  check_and_receive(rec_count,input_distribution);
  char message[MESSAGE_SIZE];
  message[0] = mcu_id;
  message[1] = 199; // 199 for ask for permission
  Serial.println("send request...");
  client.write(message,MESSAGE_SIZE);
  while(!permission_flag){
    check_and_receive(rec_count,input_distribution);
  }
  permission_flag = false; //reset the flag
}
// void check_and_receive(int* split_point,int& rec_count,byte* input_distribution,int input_size){
//   while(udp.parsePacket() >= 0){
//       Serial.println("received packet");
//       // Allocate buffer to hold incoming data
//       char packetBuffer[MESSAGE_SIZE];
//       // Read incoming packet into buffer
//       udp.read(packetBuffer, MESSAGE_SIZE);
//       byte from_which = packetBuffer[0];
//       byte length = packetBuffer[1];
//       Serial.println(rec_count);
//       Serial.print("receiving from");
//       Serial.print(from_which);

//       for(int i = 2; i < length + 2; i++){
//         int pos = 0;
//         for(int j = 0; j <= from_which; j++){
//           pos += split_point[j];
//         }
//         // if(pos < rec_count){
//         //   for(int j = input_size - 1; j > pos;j-- ){
//         //       input_distribution[j] = input_distribution[j - 1];
//         //   }          
//         // }
//         input_distribution[pos] = packetBuffer[i];
//         rec_count += 1;
//         split_point[from_which] += 1;
//         // if(rec_count == input_size) return;
//       } 
//       Serial.println(rec_count);
//   }
// }
#endif
