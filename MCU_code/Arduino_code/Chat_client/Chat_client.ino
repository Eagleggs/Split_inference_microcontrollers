#include "commu.h"

void setup() {
    Serial.begin(9600);
    setup_communication(ip1, mac1); // 使用正确的IP地址和MAC地址
    Serial.println("!!!");
}

void loop() {
    if (client[0] && client[0].connected()) {
        client[0].println("Message from MCU 1 to MCU 2");
      if (client.available() > 0) {
        // read the bytes incoming from the client:
        char thisChar = client.read();
        // echo the bytes back to the client:
        server.write(thisChar);
        // echo the bytes to the server as well:
        Serial.write(thisChar);
    }
    }

    if (client[1] && client[1].connected()) {
        client[1].println("Message from MCU 1 to MCU 3");
    }

    delay(1000);
}
