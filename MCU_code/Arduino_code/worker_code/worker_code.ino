#include "read.h"
#include "calculation.h"
#include "communication.h"
byte* input_distribution;
byte* overflow = nullptr;  // Initialize overflow pointer
bool overflow_flag = false;
int rec_count = 0;
int ino_count = 0;
void setup() {
  setup_filesys();
  {
    setup_communication(ip2,mac2); 
    byte* temp = new(std::nothrow) byte[450 * 1024];
    if(temp != nullptr) {Serial.println("success");}
    delete[] temp;
  }
  for (int j = 0; j < 53; j++) {
    Serial.print("current layer:");
    Serial.println(j);
    if(j < 52){
        if(j == 0) input_distribution = new byte[input_length[0]];
        else{
            Serial.print("rec_count is: ");
            Serial.print(rec_count);
            Serial.println("not enough inputs, receiving...");
            while(rec_count != input_length[j]){
                check_and_receive(rec_count,input_distribution);
                Serial.print("rec_count is: ");
                Serial.println(rec_count);
            }
            Serial.println("finished...");
            rec_count = 0;
        }
        int total_output_count = result_length[j];
        size_t result_size = (total_output_count > STACK_SIZE) ? (STACK_SIZE) : total_output_count;
        byte result[result_size] = { 0 };  // Initialize result array
        {
          std::vector<Weight> first_line;
          first_line = get_weights(j,prev_endpos);        
          int size = 0;
          //todo receives inputs
          // if(input_distribution == nullptr){
          //   Serial.println("nullptr!");
          // }
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
          // Serial.println("overflow");
          delete[] overflow;
        }
        input_distribution = new(std::nothrow) byte[input_length[j + 1]];
        // int mem = 10;
        // while(input_distribution == nullptr){
        //   input_distribution = new(std::nothrow) byte[input_length[j + 1] - mem];
        //   mem += 10;
        // }
        // Serial.print("space left");
        // Serial.println(input_length[j + 1] - mem);
        Serial.println("waiting for permission...");
        wait_for_permission(rec_count,input_distribution);
        Serial.println("premission granted, sending results...");
        if (j < 51) {
          char to_send[MESSAGE_SIZE];
          to_send[0] = mcu_id;
          byte send_count = 0;
          Mapping mapping;
          // Serial.println("!!!!");
          mapping = get_mapping(j + 1);
          // Serial.println("got mapping");
          int phase = mapping.count.size();
          if (overflow_flag) {
            dataFile = myfs.open("overflow.bin", FILE_READ);
            Serial.println("opened overflow");
          }
          for (int i = 0; i < phase; i++) {
            std::vector<byte> mcu_mapped = decode_u128(mapping.map[i]);    
            int padding_pos_count = 0;
            int core_count = 0;
            for (int k = 0; k < mapping.count[i]; k++) {
              if (mapping.padding_pos[i].size() > padding_pos_count && mapping.padding_pos[i][padding_pos_count] == k) {
                //send zero point to other MCUs
                // Serial.println("sending");
                to_send[send_count + 3] = mapping.zero_point[0];
                send_count += 1;
                if(send_count == MESSAGE_SIZE - 3){
                  to_send[2] = send_count;
                  sendtoMCUs(to_send,mcu_mapped,mcu_id,input_distribution,rec_count,send_count);
                  send_count = 0;
                }
                // Serial.println("send complete");
                padding_pos_count += 1;
              } else {
                if (core_count >= STACK_SIZE && overflow_flag) {
                  int count = 0;
                  to_send[send_count + 3] = read_byte(count);
                  send_count += 1;
                  if(send_count == MESSAGE_SIZE - 3){
                    to_send[2] = send_count;
                    sendtoMCUs(to_send,mcu_mapped,mcu_id,input_distribution,rec_count,send_count);
                    send_count = 0;
                  }
                } else {
                  to_send[send_count + 3] = result[core_count];
                  send_count += 1;
                  if(send_count == MESSAGE_SIZE - 3){
                    to_send[2] = send_count;
                    sendtoMCUs(to_send,mcu_mapped,mcu_id,input_distribution,rec_count,send_count);
                    send_count = 0;
                  }
                }
                core_count += 1;
              }
              //check regularly to avoid clogging
              if(rec_count < input_length[j + 1]) {
                  check_and_receive( rec_count, input_distribution);
              }
            }
            //send the rest of the data
            if(send_count != 0 ){
              to_send[2] = send_count;
              sendtoMCUs(to_send,mcu_mapped,mcu_id,input_distribution,rec_count,send_count);
              send_count = 0;
            }
          }
          if (overflow_flag) dataFile.close();
          to_send[1] = 198; //signal the end
          send_message_to_coordinator(to_send);
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
            // Serial.println(i + count);
          }
          count += LINEAR_SEGMENT;
          // Serial.println("Linear");
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
  // sendUDPMessage("1 to 2", ip2, localPort);
  // sendUDPMessage("1 to 3", ip3, localPort);
  // delay(100);
}