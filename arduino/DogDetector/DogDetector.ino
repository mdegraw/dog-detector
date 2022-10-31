#include <ESP8266WiFi.h>
#include <PubSubClient.h>
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>
#include <Adafruit_NeoPixel.h>
#include "secrets.h"

#define SCREEN_WIDTH 128 // OLED display width, in pixels
#define SCREEN_HEIGHT 64 // OLED display height, in pixels

// Declaration for an SSD1306 display connected to I2C (SDA, SCL pins)
#define OLED_RESET     -1 // Reset pin # (or -1 if sharing Arduino reset pin)
Adafruit_SSD1306 display(SCREEN_WIDTH, SCREEN_HEIGHT, &Wire, OLED_RESET);


#define PIN                 6
#define PIXELS_PER_SEGMENT  3
#define DIGITS              1

Adafruit_NeoPixel strip = Adafruit_NeoPixel(PIXELS_PER_SEGMENT * 7 * DIGITS, PIN, NEO_GRB + NEO_KHZ800);

byte segments[10] = {
  0b1111110,
  0b0011000,
  0b0110111,
  0b0111101,
  0b1011001,
  0b1101101,
  0b1101111,
  0b0111000,
  0b1111111,
  0b1111001
};

const char* ssid = WIFI_SSID;
const char* password = WIFI_PASSWORD;
const char* mqtt_server = MQTT_SERVER;
// TODO: Replace with ACKNOWLEDGE_DETECTION
const char* ACKNOWLEDGE_SIGNAL = "CANCEL_DETECTION";
int BUTTON = 12;

WiFiClient espClient;
PubSubClient client(espClient);
unsigned long lastMsg = 0;
#define MSG_BUFFER_SIZE (50)
char msg[MSG_BUFFER_SIZE];
int value = 0;
int click = 0;
int last_click = 0;
int detected_count = 0;

void setup_wifi() {
  delay(10);
  // We start by connecting to a WiFi network
  Serial.println();
  Serial.print("Connecting to ");
  Serial.println(ssid);

  WiFi.mode(WIFI_STA);
  WiFi.begin(ssid, password);

  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }

  randomSeed(micros());

  Serial.println("");
  Serial.println("WiFi connected");
  Serial.println("IP address: ");
  Serial.println(WiFi.localIP());
}


void clear7SegDisplay() {
  for (int i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, strip.Color(0, 0, 0));
  }
}

void writeDigit(int index, int value) {
  byte seg = segments[value];
  for (int i = 6; i >= 0; i--) {
    int offset = index * (PIXELS_PER_SEGMENT * 7) + i * PIXELS_PER_SEGMENT;
    uint32_t color = seg & 0x01 != 0 ? strip.Color(25, 50, 50) : strip.Color(0, 0, 0);
    for (int x = offset; x < offset + PIXELS_PER_SEGMENT; x++) {
      strip.setPixelColor(x, color);
    }
    seg = seg >> 1;
  }
}

void drawDefaultOledImage(void) {
  display.clearDisplay();

  for (int16_t i = 0; i < display.height() / 2 - 2; i += 2) {
    display.drawRoundRect(i, i, display.width() - 2 * i, display.height() - 2 * i,
                          display.height() / 4, SSD1306_WHITE);
    display.display();
    delay(1);
  }

  delay(2000);
}

void animateLoop(bool fromInner, bool clearing) {

  for (int b = fromInner ? 0 : 7; fromInner ? b < 7 : b >= 0; b += (fromInner ? 1 : -1)) {
    if (clearing) {
      clear7SegDisplay();
    }
    switch (b) {
      case 0:
        strip.setPixelColor(19, strip.Color(100, 0, 100));
        break;
      case 1:
        strip.setPixelColor(20, strip.Color(100, 0, 100));
        strip.setPixelColor(18, strip.Color(100, 0, 100));
        break;
      case 2:
        strip.setPixelColor(0, strip.Color(100, 0, 100));
        strip.setPixelColor(8, strip.Color(100, 0, 100));
        strip.setPixelColor(9, strip.Color(100, 0, 100));
        strip.setPixelColor(17, strip.Color(100, 0, 100));
        break;
      case 3:
        strip.setPixelColor(1, strip.Color(100, 0, 100));
        strip.setPixelColor(7, strip.Color(100, 0, 100));
        strip.setPixelColor(10, strip.Color(100, 0, 100));
        strip.setPixelColor(16, strip.Color(100, 0, 100));
        break;
      case 4:
        strip.setPixelColor(2, strip.Color(100, 0, 100));
        strip.setPixelColor(6, strip.Color(100, 0, 100));
        strip.setPixelColor(11, strip.Color(100, 0, 100));
        strip.setPixelColor(15, strip.Color(100, 0, 100));
        break;
      case 5:
        strip.setPixelColor(3, strip.Color(100, 0, 100));
        strip.setPixelColor(5, strip.Color(100, 0, 100));
        strip.setPixelColor(12, strip.Color(100, 0, 100));
        strip.setPixelColor(14, strip.Color(100, 0, 100));
        break;
      case 6:
        strip.setPixelColor(4, strip.Color(100, 0, 100));
        strip.setPixelColor(13, strip.Color(100, 0, 100));
        break;
      default: break;
    }
    strip.show();
    delay(100);
  }
}

void read_click() {
  click = digitalRead(BUTTON);

  if (last_click != click) {
    last_click = click;
    if (click == HIGH) {
        Serial.println("Acknowledge Detection");
        char buffer[256];
        detected_count = 0;
        display.clearDisplay();
        drawDefaultOledImage();
        client.publish(DOG_DETECTED_ACKNOWLEDGE_TOPIC, buffer, ACKNOWLEDGE_SIGNAL);
    }
  }
}

void callback(char* topic, byte* payload, unsigned int length) {
  if (strcmp(topic, DOG_DETECTED_STREAM_TOPIC) == 0) {
    display.clearDisplay();
    display.drawBitmap(0, 0, payload, 128, 64, SSD1306_WHITE);
    display.display();
  } else if (strcmp(topic, DOG_DETECTED_TOPIC) == 0) {
    detected_count += 1;
    Serial.println("\n\n\n\nDOG DETECTED_COUNT");
    Serial.println(detected_count);
  } else if (strcmp(topic, DOG_DETECTED_STREAM_END_TOPIC) == 0) {
    Serial.println("\n\n\n\nDOG_DETECTED_STREAM_END_TOPIC");
    drawDefaultOledImage();
  }
}

void reconnect() {
  // Loop until we're reconnected
  while (!client.connected()) {
    Serial.print("Attempting MQTT connection...");
    // Create a random client ID
    String clientId = "ESP8266Client-";
    clientId += String(random(0xffff), HEX);
    // Attempt to connect
    if (client.connect(clientId.c_str())) {
      Serial.println("connected");
      client.setBufferSize(16384);
      client.subscribe(DOG_DETECTED_TOPIC);
      client.subscribe(DOG_DETECTED_STREAM_TOPIC);
      client.subscribe(DOG_DETECTED_STREAM_END_TOPIC);
    } else {
      Serial.print("failed, rc=");
      Serial.print(client.state());
      Serial.println(" try again in 5 seconds");
      // Wait 5 seconds before retrying
      delay(5000);
    }
  }
}

void setup() {
  Serial.begin(115200);
  pinMode(BUTTON, INPUT);
  // SSD1306_SWITCHCAPVCC = generate display voltage from 3.3V internally
  if (!display.begin(SSD1306_SWITCHCAPVCC, 0x3C)) {
    Serial.println(F("SSD1306 allocation failed"));
    for (;;); // Don't proceed, loop forever
  }
  setup_wifi();
  client.setServer(mqtt_server, 1883);
  client.setCallback(callback);

  display.clearDisplay();
  drawDefaultOledImage();
  display.display();
}

void loop() {
  read_click();

  if (!client.connected()) {
    reconnect();
  }
  client.loop();
}
