import serial
import time

# Configure serial port (change COMx to match your Arduino's serial port)
ser = serial.Serial('COM3', 9600, timeout=1)  # Adjust baud rate and port as needed

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
    try:
        while True:
            send_data_to_arduino("Hello from Python!")
            time.sleep(1)  # Wait for 1 second
            read_data_from_arduino()
    except KeyboardInterrupt:
        print("Program terminated by user.")
    finally:
        ser.close()  # Close the serial port