# Dog Detector ESP8266

## Instructions
* Create `secrets.h` in `arduino/DogDetector`
* Add the following env variables to `secrets.h`
    ```c
    #define WIFI_SSID ""
    #define WIFI_PASSWORD ""
    #define MQTT_SERVER ""
    #define DOG_DETECTED_TOPIC ""
    ```
* Install the required libraries in the Arduino IDE ([PubSubClient](https://pubsubclient.knolleary.net/), [Adafruit_SSD1306](https://github.com/adafruit/Adafruit_SSD1306), [Adafruit_GFX](https://github.com/adafruit/Adafruit-GFX-Library))
* Connect SSD1306 display to ESP8266
![alt text](https://i0.wp.com/randomnerdtutorials.com/wp-content/uploads/2019/05/ESP8266_oled_display_wiring.png?w=828&quality=100&strip=all&ssl=1 "1306 wiring diagram")
* Compile and upload to ESP8266