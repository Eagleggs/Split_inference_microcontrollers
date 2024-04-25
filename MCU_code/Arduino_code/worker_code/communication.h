//200: permission granted 199: ask for permission 198:send complete 197:ack
#ifndef COMMUNICATION_H
#define COMMUNICATION_H

#include <SPI.h>
#include <NativeEthernet.h>
#define MESSAGE_SIZE 1400
const int reserve_bytes = 6; //|from which|to which/message type|length * 4|
const int num_mcu = 3;
const byte mcu_id = 0;
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
  Ethernet.setSocketSize(8 * 1024);
  Ethernet.setSocketNum(1);
  Ethernet.begin(mac,ip);// Change to mac2 and ip2 for MCU2, mac3 and ip3 for MCU3
  Serial.println("connecting...");
  while (!client.connect(server, serverport)) {} //connect to server
  while (!client.available()) {} //read a byte from server to indicate communication established
  char c = client.read();
  Serial.println("connection established!");
}
bool wait_for_ack(){
  while(!client.available()){};
  char message[3];
  client.readBytes(message,3);
  if(message[1] != 197){
    return true;
  }
  return false;
}
bool send_message_to_coordinator(const char* message){
  if(client.write(message,MESSAGE_SIZE) == 0) return false;
  if(wait_for_ack()){
    Serial.println("ack message wrong, stop executing...");
    while(1){};
  };
  return true;  
}

void send_ack(){
  char message[3];
  message[0] = mcu_id;
  message[1] = 197;
  client.write(message,MESSAGE_SIZE);
}

void sendtoMCUs(char* message, std::vector<byte>& MCUs,const byte cur_mcu,byte* cur_input,int& rec_count,int& send_count){
  message[1] = 0;
  for(byte m : MCUs){
    if(m == cur_mcu){
      for(int i = reserve_bytes - 1; i <send_count + reserve_bytes - 1; i++ ){
        cur_input[rec_count] = message[i];
        rec_count += 1;
      }
    }
    else{
      message[1] |= 1 << m;
    }     
  }
  if(message[1] != 0){
    send_message_to_coordinator(message);    
  }
}
void check_and_receive(int& rec_count,byte* input_distribution){
  int count = 0;
  if(client.available()){  
    byte buffer[MESSAGE_SIZE];
    client.readBytes(buffer,MESSAGE_SIZE);
    if(buffer[1] == 200){ 
      permission_flag = true;
    }else{
      byte from_which = buffer[0];
      byte to_which = buffer[1]; 
      int length = 0;
      memcpy(&length, buffer + 2, sizeof(int));
      if(to_which != mcu_id){
        Serial.println("received wrong message!! Stop executing...");
        Serial.println((byte) to_which);
        while(1){};
      }
      for(int i = 0; i < length; i++){
        input_distribution[rec_count] = buffer[i + reserve_bytes];
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
void write_length(byte* message,int length){
    char intBytes[sizeof(int)];
    memcpy(intBytes, &length, sizeof(int));
    memcpy(message + 2, intBytes, sizeof(int));
}
#endif
