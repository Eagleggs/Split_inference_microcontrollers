#include "read.h"
#include "calculation.h"
#include "communication.h"
int input_length[53] = { 151875, 138702, 401408, 200704, 408617, 301056, 75264, 161481, 451584, 75264, 155961, 112896, 25088, 57609, 150528, 25088, 57609, 150528, 25088, 53833, 37632, 12544, 32777, 75264, 12544, 32777, 75264, 12544, 32777, 75264, 12544, 32777, 75264, 18816, 49161, 112896, 18816, 49161, 112896, 18816, 43209, 28224, 7840, 25929, 47040, 7840, 25929, 47040, 7840, 25929, 47040, 15680, 1280 };
int result_length[53] = {133804, 133804, 66903, 401409, 100353, 25089, 150529, 150529, 25089, 150529, 37633, 8364, 50177, 50177, 8364, 50177, 50177, 8364, 50177, 12545, 4183, 25089, 25089, 4183, 25089, 25089, 4183, 25089, 25089, 4183, 25089, 25089, 6273, 37633, 37633, 6273, 37633, 37633, 6273, 37633, 9409, 2615, 15681, 15681, 2615, 15681, 15681, 2615, 15681, 15681, 5228, 20908,334};
byte* input_distribution;
const byte mcu_id = 0;
byte* overflow = nullptr;  // Initialize overflow pointer
bool overflow_flag = false;
void setup() {
  setup_filesys();
  {
     setup_communication(ip1,mac1); 
    byte* temp = new(std::nothrow) byte[450 * 1024];
    if(temp != nullptr) {Serial.println("success");}
    delete[] temp;
  }
  for (int j = 0; j < 53; j++) {
    if(j < 52){
        if(j == 0) input_distribution = new byte[input_length[0]];
        int total_output_count = result_length[j];
        size_t result_size = (total_output_count > STACK_SIZE) ? (STACK_SIZE) : total_output_count;
        byte result[result_size] = { 0 };  // Initialize result array
        {
          std::vector<Weight> first_line;
          first_line = get_weights(j,prev_endpos);
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
            overflow = new byte[total_output_count - STACK_SIZE];  // Allocate memory for overflow
            Serial.println(total_output_count - STACK_SIZE);

          } else {
            overflow_flag = false;
          }
          // Determine the size of the result array based on the condition
          // Call the distributed_computation function with appropriate arguments
          distributed_computation(first_line, input_distribution, result, overflow, input_length[j]);
          delete[] input_distribution;
        }
        if (overflow_flag) {
          otf(overflow, total_output_count - STACK_SIZE);
          Serial.println("overflow");
          delete[] overflow;
        }
        input_distribution = new(std::nothrow) byte[input_length[j + 1]];
        if (j < 51) {
          Mapping mapping;
          mapping = get_mapping(j + 1);
          // Serial.println("got mapping");
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
      else if(j >= 52 ){
        byte result[result_length[j]] = {0};
        int count = 0;
        reading_weight = true;
        while(reading_weight){
          byte segment[LINEAR_SEGMENT] = {0};
          std::vector<Weight> weight = get_weights(j,prev_endpos);
          for (int i = 0; i < input_length[j]; i++) {
            input_distribution[i] = i % 255;
          }
          distributed_computation(weight, input_distribution, segment, overflow, input_length[j]);
          for(int i = 0; i < LINEAR_SEGMENT; i++){
            result[i + count] = segment[i];
            if(i + count >= result_length[j]) break;
            Serial.println(i + count);
          }
          count += LINEAR_SEGMENT;
          Serial.println("Linear");
        }
      }      
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
  // Check if data is available to be read
  int packetSize = udp.parsePacket();
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
  char packet[20];
  byte to_send[20] = {48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67};
  for(int i = 0; i < 20; i++){
    packet[i] = (char)to_send[i];
  }
  // sendUDPMessage("1 to 2", ip2, localPort);
  // sendUDPMessage("1 to 3", ip3, localPort);
  // delay(100);
}