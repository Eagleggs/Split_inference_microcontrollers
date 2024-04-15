import serial
import time
import json

# Configure serial port (change COMx to match your Arduino's serial port)
ser = serial.Serial('COM5', 9600, timeout=1)  # Adjust baud rate and port as needed


def send_data_to_arduino(data):
    # Send data to Arduino
    ser.write(data.encode())
    print("Data sent to Arduino:", data)


def read_data_from_arduino():
    # Read data from Arduino
    data = ser.readline().decode().strip()
    while data:
        print("Data received from Arduino:", data)
        data = ser.readline().decode().strip()
    return data


# Main program
if __name__ == "__main__":
    file = "../pc_code/Simulation/Simu_q/worker_0.json"
    with open(file, 'r') as json_file:
        for line in json_file:
            data = json.loads(line)
            for weight in data['weights']:
                weight_to_send = ""
                d = weight['data']
                # print(d)
                weight_to_send += str(len(d)) + ' '
                # send_data_to_arduino(str(len(d)) + ' ')
                for i in d:
                    try:
                        weight_to_send += str(i) + ' '
                        # send_data_to_arduino(str(i) + ' ')
                    except KeyboardInterrupt:
                        print("Program terminated by user.")
                weight_to_send = weight_to_send[:-1]
                weight_to_send += '!'
                send_data_to_arduino(weight_to_send)
                # read_data_from_arduino()
                bias = weight['bias']
                send_data_to_arduino(str(bias) + '!')
                # read_data_from_arduino()
                which_kernel = weight['which_kernel']
                send_data_to_arduino(str(which_kernel) + '!')
                # read_data_from_arduino()
                count = weight['count']
                send_data_to_arduino(str(count) + '!')
                # read_data_from_arduino()
                if len(weight['start_pos_in']) != 0:
                    start_pos_int = str(weight['start_pos_in'][0]) + ' ' + str(weight['start_pos_in'][1]) + ' ' + str(
                        weight['start_pos_in'][2]) + '!'
                    send_data_to_arduino(start_pos_int)
                else:
                    send_data_to_arduino('!')
                info = weight['info']
                if 'Convolution' in info:
                    t = info['Convolution']
                    o_pg = str(t['o_pg'])
                    i_pg = str(t['i_pg'])
                    s = str(t['s'][0]) + ' ' + str(t['s'][1])
                    k = str(t['k'][0]) + ' ' + str(t['k'][1])
                    i_n = str(t['i'][0]) + ' ' + str(t['i'][1]) + ' ' + str(t['i'][2])
                    o = str(t['o'][0]) + ' ' + str(t['o'][1]) + ' ' + str(t['o'][2])
                    to_send = 'C ' + o_pg + ' ' + i_pg + ' ' + s + ' ' + k + ' ' + i_n + ' ' + o + '!'
                    send_data_to_arduino(to_send)
                else:
                    t = info['Linear']
                    b_in = str(t['b_in'])
                    c_in = str(t['c_in'])
                    b_out = str(t['b_out'])
                    c_out = str(t['c_out'])
                    to_send = 'L' + ' ' + b_in + ' ' + c_in + ' ' + b_out + ' ' + c_out + '!'
                    send_data_to_arduino(to_send)

                zero_points = str(weight['zero_points'][0]) + ' ' + str(weight['zero_points'][1]) + ' ' + str(
                    weight['zero_points'][2])
                m = str(weight['m'])
                s_out = str(weight['s_out'])
                to_send = zero_points + ' ' + m + ' ' + s_out + '!'
                send_data_to_arduino(to_send)
                # read_data_from_arduino()
                # time.sleep(0.5)  # Wait for 1 second

            print("send line complete")
            send_data_to_arduino('!')
        send_data_to_arduino('!')
        read_data_from_arduino()


print('---------------')
