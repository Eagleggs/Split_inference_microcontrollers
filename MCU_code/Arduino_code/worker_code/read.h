#ifndef READ_H
#define READ_H
#include <iterator>
#include <vector>
#include "worker_struct.h"
#include "coordinator_struct.h"
#include "C:\Users\Lu JunYu\Documents\Arduino\download\filesys.h"
int input_length[53] = {151875, 138931, 401408, 200704, 408613, 301056, 75264, 161480, 451584, 75264, 155957, 112896, 25088, 57608, 150528, 25088, 57608, 150528, 25088, 53829, 37632, 12544, 32776, 75264, 12544, 32776, 75264, 12544, 32776, 75264, 12544, 32776, 75264, 18816, 49160, 112896,18816, 49160, 112896, 18816, 43205, 28224, 7840, 25928, 47040, 7840, 25928, 47040, 7840, 25928, 47040, 15680,1280};
int result_length[53] = {133803, 133803, 66902, 401408, 100352, 25088, 150528, 150528, 25088, 150528, 37632, 8363, 50176, 50176, 8363, 50176, 50176, 8363, 50176, 12544,4182, 25088, 25088, 4182, 25088, 25088, 4182, 25088, 25088, 4182, 25088, 25088,6272, 37632, 37632, 6272, 37632, 37632, 6272, 37632, 9408, 2614, 15680, 15680, 2614, 15680, 15680, 2614, 15680, 15680, 5227, 20907,334};
int lines[] = { 1380, 2544, 3264, 6696, 9897, 11553, 17041, 21794, 23882, 29370, 34123, 36907, 44707, 51012, 54372, 62172, 68477, 71837, 79637, 85942, 92102, 111710, 124223, 134607, 154215, 166728, 177112, 196720, 209233, 219617, 239225, 251738, 267314, 302826, 321547, 343459, 378971, 397692, 419604, 455116, 473837, 509693, 589301, 620438, 677030, 756638, 787775, 844367, 923975, 955112, 1068296, 1242920, 1683132 };
int coor_lines[] ={1849, 21554, 21607, 21660, 50669, 50722, 50775, 94760, 94813, 94866, 116771,116824, 116877, 146782, 146835, 146888, 176793, 176846, 176899, 191700, 191753,191806, 222735, 222788, 222841, 253770, 253823, 253876, 284805, 284858, 284911,315840, 315893, 315946, 362235, 362288, 362341, 408630, 408683, 408736, 431217,431270, 431323, 472492, 472545, 472598, 513767, 513820, 513873, 555042, 555095,555148};

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
#endif