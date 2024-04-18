#include "read.h"
#include "calculation.h"
#include <NativeEthernet.h>
byte mac[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xEB };  // MAC address
IPAddress serverIP(192, 168, 1, 177);                 // IP address of the server
IPAddress ip(192, 168, 1, 175);
IPAddress myDns(192, 168, 1, 1);
IPAddress gateway(192, 168, 1, 1);
IPAddress subnet(255, 255, 0, 0);
EthernetClient client;  // Create a client object
int input_length[53] = { 151875, 138702, 401408, 200704, 408617, 301056, 75264, 161481, 451584, 75264, 155961, 112896, 25088, 57609, 150528, 25088, 57609, 150528, 25088, 53833, 37632, 12544, 32777, 75264, 12544, 32777, 75264, 12544, 32777, 75264, 12544, 32777, 75264, 18816, 49161, 112896, 18816, 49161, 112896, 18816, 43209, 28224, 7840, 25929, 47040, 7840, 25929, 47040, 7840, 25929, 47040, 15680, 1280 };
int result_length[53] = {133804, 133804, 66903, 401409, 100353, 25089, 150529, 150529, 25089, 150529, 37633, 8364, 50177, 50177, 8364, 50177, 50177, 8364, 50177, 12545, 4183, 25089, 25089, 4183, 25089, 25089, 4183, 25089, 25089, 4183, 25089, 25089, 6273, 37633, 37633, 6273, 37633, 37633, 6273, 37633, 9409, 2615, 15681, 15681, 2615, 15681, 15681, 2615, 15681, 15681, 5228, 20908,334};
byte* input_distribution;
const int num_mcu = 3;
const byte mcu_id = 0;
byte* overflow = nullptr;  // Initialize overflow pointer
bool overflow_flag = false;
void setup() {
  setup_filesys();
  // Ethernet.begin(mac, ip, myDns, gateway, subnet);
  // if (client.connect(serverIP, 23)) { // Connect to the server
  // }
  for (int j = 0; j < 53; j++) {
    if(j == 0) input_distribution = new byte[input_length[0]];
    int total_output_count = result_length[j];
    size_t result_size = (total_output_count > STACK_SIZE) ? (STACK_SIZE) : total_output_count;
    byte result[result_size] = { 0 };  // Initialize result array
    {
      std::vector<Weight> first_line = get_weights(j); 
      int size = 0;
      //todo receives inputs
      if(input_distribution == nullptr){
        Serial.println("nullptr!");
      }
      for (int i = 0; i < input_length[j]; i++) {
        input_distribution[i] = i % 255;
      }
      ////////////////////////////
      // Check if the total output count exceeds the threshold
      if (total_output_count > STACK_SIZE) {
        overflow_flag = true;
        Serial.println(total_output_count - STACK_SIZE);
        overflow = new byte[total_output_count - STACK_SIZE];  // Allocate memory for overflow
      } else {
        overflow_flag = false;
      }
      // Determine the size of the result array based on the condition
      // Call the distributed_computation function with appropriate arguments
      distributed_computation(first_line, input_distribution, result, overflow, input_length[j]);
    }
    delete[] input_distribution;
    if (overflow_flag) {
      Serial.println("overflow");
      otf(overflow, total_output_count - STACK_SIZE);
      delete[] overflow;
    }
    if(j < 52){
      input_distribution = new byte[input_length[j + 1]];      
    }
    if (j < 51) {
      Mapping mapping;
      mapping = get_mapping(j + 1);
      int phase = mapping.count.size();
      int remain_count = 0;
      for (int i = 0; i < phase; i++) {  //find how many result should be reserved for next calculation
        std::vector<byte> mcu_mapped = decode_u128(mapping.map[i]);
        if (std::find(mcu_mapped.begin(), mcu_mapped.end(), mcu_id) != mcu_mapped.end()) {
          remain_count += mapping.count[i];
        }
      }
      if (overflow_flag) {
        dataFile = myfs.open("overflow.bin", FILE_READ);
      }
      for (int i = 0; i < phase; i++) {
        std::vector<byte> mcu_mapped = decode_u128(mapping.map[i]);
        int padding_pos_count = 0;
        int core_count = 0;
        for (int j = 0; j <= mapping.count[i]; j++) {
          if (mapping.padding_pos[i].size() > padding_pos_count && mapping.padding_pos[i][padding_pos_count] == j) {
            //send zero point to other MCUs
            padding_pos_count += 1;
          } else {
            if (core_count >= STACK_SIZE && overflow_flag) {
              int count = 0;
              byte to_send = read_byte(count);
              //send(overflow[core_count - STACK_SIZE])
            } else {
              //send(result[core_count]);
            }
            core_count += 1;
          }
        }
      }
      if (overflow_flag) dataFile.close();
    }
    ///////////////////////////
  }
}
void loop() {
  if (Serial.available()) {
    char rr;
    rr = Serial.read();
    switch (rr) {
      case 'l': listFiles(); break;
      case 'e': eraseFiles(); break;
      // case 'x': stopLogging(); break;
      case 'd': dumpLog(); break;
      case '\r':
      case '\n':
      case 'h': menu(); break;
    }
    while (Serial.read() != -1)
      ;  // remove rest of characters.
  }
}
