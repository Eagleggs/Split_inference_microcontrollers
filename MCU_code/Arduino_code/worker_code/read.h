#ifndef READ_H
#define READ_H
#include <iterator>
#include <vector>
#include "worker_struct.h"
#include "coordinator_struct.h"
#include "C:\Users\Lu JunYu\Documents\Arduino\download\filesys.h"
int input_length[53] = {151875, 138699, 401408, 200704, 408604, 301056, 75264, 161471, 451584, 75264, 155948, 112896, 25088, 57599, 150528, 25088, 57599, 150528, 25088, 53820, 37632, 12544, 32767, 75264, 12544,32767, 75264, 12544, 32767, 75264, 12544, 32767, 75264, 18816, 49151, 112896, 18816, 49151, 112896, 18816, 43196, 28224, 7840, 25919, 47040, 7840, 25919, 47040, 7840, 25919, 47040, 15680,1280};
int result_length[53] = {133801, 133801, 66899, 401407, 100351, 25087, 150527, 150527, 25087, 150527, 37631, 8361, 50175, 50175, 8361, 50175, 50175, 8361, 50175, 12543, 4179, 25087, 25087, 4179, 25087, 25087, 4179, 25087, 25087, 4179, 25087, 25087, 6271, 37631, 37631, 6271, 37631, 37631, 6271, 37631, 9407, 2611, 15679, 15679, 2611, 15679, 15679, 2611, 15679, 15679, 5225, 20905,332};
int lines[] = { 1265, 2332, 3052, 6380, 9484, 10956, 16332, 20988, 22844, 28220, 32876, 35428, 43108, 49316, 52396, 60076, 66284, 69364, 77044, 83252, 89412, 108868,121284, 131668, 151124, 163540, 173924, 193380, 205796, 216180, 235636, 248052,263156, 298484, 317108, 338356, 373684, 392308, 413556, 448884, 467508, 503364,582724, 613764, 670356, 749716, 780756, 837348, 916708, 947748, 1059884, 1234100, 1670358 };
int coor_lines[] ={981, 20214, 20303, 20392, 48885, 48974, 49063, 92752, 92841, 92930, 114543,114632, 114721, 144442, 144531, 144620, 174341, 174430, 174519, 189140, 189229,189318, 220119, 220208, 220297, 251098, 251187, 251276, 282077, 282166, 282255,313056, 313145, 313234, 359395, 359484, 359573, 405734, 405823, 405912, 428269,428358, 428447, 469516, 469605, 469694, 510763, 510852, 510941, 552010, 552099,552188};


bool reading_weight = true; 
int prev_endpos = 0;
const int LINEAR_SEGMENT  = 300;
int littleEndianToInt(const char* bytes) {
  // Interpret the reversed buffer as an integer using pointer
  int result;
  memcpy(&result, bytes, sizeof(int));
  return result;
}
float littleEndianToFloat(const char* bytes) {
  // Interpret the reversed buffer as a float using pointer
  float result;
  memcpy(&result, bytes, sizeof(float));
  return result;
}
byte read_byte(int& count) {
  byte res = 0;
  if (dataFile) {
    res = static_cast<byte>(dataFile.read());
    count += 1;
  }
  return res;
}
int read_int(int& count) {
  int res = 0;
  char buffer[4];
  if (dataFile) {
    dataFile.readBytes(buffer, 4);
    res = littleEndianToInt(buffer);
    count += 4;
  }
  return res;
}
float read_float(int& count) {
  float res = 0;
  char buffer[4];
  if (dataFile) {
    dataFile.readBytes(buffer, 4);
    res = littleEndianToFloat(buffer);
    count += 4;
  }
  return res;
}
std::vector<Weight> get_weights(int line_number,int& prev_endpos) {
  // Serial.println(prev_endpos);
  std::vector<Weight> res;
  char filename[20] = "datalog.bin";
  dataFile = myfs.open(filename, FILE_READ);
  if (dataFile) {
    reading_weight = true;
    int start = line_number == 0 ? 0 : lines[line_number - 1];
    if(prev_endpos != 0) {start = prev_endpos;}
    dataFile.seek(start);
    int count = start;
    int weight_size = 0;
    int weight_count = 0;
    while (count < lines[line_number]) {
      if (count == start) {
        Weight a;
        weight_size = read_int(count);
        for (int i = 0; i < weight_size; i++) {
          a.data.push_back(read_byte(count));
        }
        a.bias = read_int(count);
        a.which_kernel = read_int(count);
        a.count = read_int(count);
        if (line_number != 52) {  //handel linear may change later
          for (int i = 0; i < 3; i++) {
            a.start_pos_in[i] = read_int(count);
          }
        }
        if (read_byte(count) == 0) {
          a.i.type = Type::Convolution;
          a.i.c_info.o_pg = read_int(count);
          a.i.c_info.i_pg = read_int(count);
          for (int i = 0; i < 2; i++) {
            a.i.c_info.s[i] = (read_int(count));
          }
          for (int i = 0; i < 2; i++) {
            a.i.c_info.k[i] = (read_int(count));
          }
          for (int i = 0; i < 3; i++) {
            a.i.c_info.i[i] = (read_int(count));
          }
          for (int i = 0; i < 3; i++) {
            a.i.c_info.o[i] = (read_int(count));
          }
        } else {
          a.i.type = Type::Linear;
          a.i.l_info.b_in = read_byte(count);
          a.i.l_info.c_in = read_int(count);
          a.i.l_info.b_out = read_byte(count);
          a.i.l_info.c_out = read_int(count);
        }
        for (int i = 0; i < 3; i++) {
          a.zero_points[i] = (read_byte(count));
        }
        a.m = read_float(count);
        a.s_out = read_float(count);
        res.push_back(a);
        start = count;
        weight_count += 1;
        if(a.i.type == Type::Linear){
          if(count == lines[line_number]){
            dataFile.close();
            reading_weight = false;
            prev_endpos = 0;
            return res;
          }
          if(weight_count >= LINEAR_SEGMENT) {
            prev_endpos = count;
            return res;
          }
        }
      } else {
        Serial.println("data not alligned! please check the code");
        break;
      }
    }
    dataFile.close();
    reading_weight = false;
  } else (Serial.println("weights data not available!"));

  return res;
}
Mapping get_mapping(int line_number) {
  char filename[20] = "Coordinator.bin";
  dataFile = myfs.open(filename, FILE_READ);
  Mapping mapping;
  if (dataFile) {  

    int start = line_number == 0 ? 0 : coor_lines[line_number - 1];
    dataFile.seek(start);
    int count = start;
    int phase = 0;
    phase = read_int(count);
    for (int i = 0; i < phase; i++) {
      mapping.count.push_back(read_int(count));
    }
    for (int i = 0; i < phase; i++) {
      std::vector<byte> temp;
      for (int j = 0; j < 16; j++) {
        temp.push_back(read_byte(count));
      }
      mapping.map.push_back(temp);
    }
    for (int i = 0; i < phase; i++) {
      int len = read_int(count);
      std::vector<int> temp(len);
      // temp.reserve(len * 4);
      for (int j = 0; j < len; j++) {
        temp[j] = (read_int(count));
      }
      mapping.padding_pos.push_back(temp);
    }
    byte len = read_byte(count);
    if (len > 0) {
      for (int i = 0; i < len; i++) {
        std::vector<int> temp;
        for (int j = 0; j < 3; j++) {
          temp.push_back(read_int(count));
        }
        mapping.end_pos.push_back(temp);
      }
    }
    for (int i = 0; i < 3; i++) {
      mapping.zero_point[i] = read_int(count);
    }
    for (int i = 0; i < 3; i++) {
      mapping.scale[i] = read_float(count);
    }
    if(count != coor_lines[line_number]){
      Serial.println("data not alligned!");
    }
    dataFile.close();
  } else {
    Serial.println("Coordinator data not available!");
  }
  return mapping;
}
std::vector<byte> decode_u128(const std::vector<byte>& input) {
    std::vector<byte> next_mcus;
    unsigned int offset = 0;

    for (const auto& t : input) {
        for (int i = 0; i < 8; ++i) {
            if ((t >> i) & 0b1) {
                next_mcus.push_back(offset + i);
            }
        }
        offset += 8;
    }
    return next_mcus;
}
void otf(byte* overflow,int size){
  char filename[20] = "overflow.bin";
  if(myfs.exists(filename)){
    myfs.remove(filename);
  }
  dataFile = myfs.open(filename,FILE_WRITE);
  for(int i = 0; i < size; i++){
    write_byte(overflow[i]);
  }
  dataFile.close();
}
void handle_residual(byte* input,int size,int layer_id,std::vector<std::vector<int>>& connections,std::vector<std::vector<byte>>& zps,std::vector<std::vector<float>>& scales){
  if(!connections.empty()){
    if(connections[0][0] == layer_id){
      char filename[20] = "residual.bin";
      if(myfs.exists(filename)){
        myfs.remove(filename);
      }
      dataFile = myfs.open(filename,FILE_WRITE);
      for(int i = 0; i < size; i++){
        write_byte(input[i]);
      }
      dataFile.close();
    }
    else if (connections[0][1] == layer_id){
      dataFile = myfs.open("residual.bin",FILE_READ);
      int count = 0;
      for(int i = 0; i < size; i++){
        int temp = int (round((float(input[i] - zps[0][1]) * scales[0][1] + float(read_byte(count) - zps[0][0]) * scales[0][0]) / float (scales[0][2]) + float (zps[0][2])));
        if(temp > 255) input[i] = 255;
        else if(temp < 0) input[i] = 0;
        else{
          input[i] = temp;
        }
      }
      connections.erase(connections.begin());
      zps.erase(zps.begin());
      scales.erase(scales.begin());
      dataFile.close();
      if(!connections.empty() && connections[0][0] == layer_id){
        char filename[20] = "residual.bin";
        if(myfs.exists(filename)){
          myfs.remove(filename);
        }
        dataFile = myfs.open(filename,FILE_WRITE);
        for(int i = 0; i < size; i++){
          write_byte(input[i]);
        }
        dataFile.close();
      }
    }
  }
}
#endif