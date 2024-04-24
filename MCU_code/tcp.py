import socket
import select
import time





# PC's IP address and port
message_size = 1400
num_mcu = 3
pc_ip = "169.254.71.125"  # Replace with PC's IP address
pc_port = 8080  # Replace with PC's port number
ip1 = "169.254.71.124"
ip2 = "169.254.71.123"
ip3 = "169.254.71.122"
# Create a TCP/IP socket
server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

# Bind the socket to the address and port
server_socket.bind((pc_ip, pc_port))

# Listen for incoming connections
server_socket.listen(num_mcu)
print("Waiting for connection...")
sockets = [None] * num_mcu
addresses = [None] * num_mcu
which = -1
for count in range(num_mcu):
    # Accept a connection
    client_socket, client_address = server_socket.accept()
    client_socket.setblocking(0)  # Set socket to non-blocking mode
    print(str(client_address[0]))
    if str(client_address[0]) == ip1:
        which = 0
    if str(client_address[0]) == ip2:
        which = 1
    if str(client_address[0]) == ip3:
        which = 2
    sockets[which] = client_socket
    addresses[which] = client_address
    print(f"Connected to Arduino {which} at:", client_address)
for s in sockets:
    s.sendall(b'\x41')  # signal of connection success
print("connection established!")
working = False
pending = []
cur = 0
layer_levels = [0] * num_mcu
last_send_time = time.time()
ep = 0
data_to_send = bytearray(message_size)
from_which = -1
input_flag = False
def wait_for_ack(sock, message_size):
    while True:
        print("waiting for ack")
        socketslist = [sock]
        read, _, _ = select.select(socketslist, [], [])
        if len(read) > 0:
            for s in read:
                message = bytearray(s.recv(message_size))
                if message[1] == 197:
                    print("ack received!")
                    return
                elif message[1] == 199:
                    print(f"received request from {message[0]}")
                    pending.append([layer_levels[message[0]], message[0]])
                    pending.sort()
                    layer_levels[message[0]] += 1
                    print(pending)

def send_ack(sock, data):
    print("sending ack")
    data[1] = 197
    sock.sendall(data)

try:
    while True:
        readable, _, _ = select.select(sockets, [], [])  # Select sockets ready to read
        for sock in readable:
            for count in range(num_mcu):
                if sock == sockets[count]:
                    try:
                        # time.sleep(0.01)
                        received_data = sock.recv(message_size)
                    except:
                        continue
                        # received_data = sock.recv(message_size)
                    received_data = bytearray(received_data)
                    if received_data[1] != 199:
                        send_ack(sock, received_data.copy())
                    byte_array = bytes(received_data)
                    print(f"len of data is:{len(byte_array)}")
                    from_which = received_data[0]
                    if received_data[1] == 199:
                        print(f"received request from{from_which}")
                        pending.append([layer_levels[from_which], from_which])
                        pending.sort()
                        print(pending)
                        layer_levels[from_which] += 1
                        if not working:
                            next = pending[0][1]
                            if next == ep:
                                working = True
                                ep += 1
                                if ep == num_mcu:
                                    ep = 0
                                received_data = bytearray(received_data)
                                received_data[1] = 200
                                print(f"sending permission to {next}")
                                sockets[next].sendall(received_data)
                                wait_for_ack(sockets[next], message_size)
                    elif received_data[1] == 198:
                        print(pending)
                        print(ep)
                        pending.pop(0)
                        working = False
                        received_data[1] = 200
                        if len(pending) != 0 and ep == pending[0][1]:
                            ep += 1
                            if ep == num_mcu:
                                ep = 0
                            next = pending[0]
                            print(pending)
                            print(f"next: {next}")
                            working = True
                            print(f"sending permission to {next}")
                            sockets[next[1]].sendall(received_data)
                            wait_for_ack(sockets[next[1]], message_size)
                    else:
                        data_to_send = received_data
                        to_which = data_to_send[1]
                        print(f"Received from Arduino{from_which} to {to_which}:")
                        # print("data:", byte_array)
                        if data_to_send:
                            match to_which:
                                case 0:
                                    print("send to 0")
                                    try:
                                        sockets[0].sendall(data_to_send)
                                        wait_for_ack(sockets[0], message_size)
                                    except BlockingIOError:
                                        time.sleep(10)
                                        sockets[0].sendall(data_to_send)

                                case 1:
                                    print("send to 1")
                                    try:
                                        sockets[1].sendall(data_to_send)
                                        wait_for_ack(sockets[1], message_size)
                                    except BlockingIOError:
                                        time.sleep(10)
                                        sockets[1].sendall(data_to_send)

                                case 2:
                                    print("send to 2")
                                    try:
                                        sockets[2].sendall(data_to_send)
                                        wait_for_ack(sockets[2], message_size)
                                    except BlockingIOError:
                                        time.sleep(10)
                                        sockets[2].sendall(data_to_send)
except KeyboardInterrupt:
    print("Closing connection")
