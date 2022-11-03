// neopixel 7 segment code modified from here https://github.com/mattiasjahnke/arduino-projects/blob/master/neo-segment/neo-segment.ino
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
#define OLED_RESET -1 // Reset pin # (or -1 if sharing Arduino reset pin)
Adafruit_SSD1306 display(SCREEN_WIDTH, SCREEN_HEIGHT, &Wire, OLED_RESET);


#define PIN 9
#define PIXELS_PER_SEGMENT 2
#define DIGITS 1
Adafruit_NeoPixel strip
    = Adafruit_NeoPixel(PIXELS_PER_SEGMENT * 7 * DIGITS, PIN, NEO_GRB + NEO_KHZ800);

byte segments[10] = { 0b1111110, 0b0011000, 0b0110111, 0b0111101, 0b1011001, 0b1101101, 0b1101111,
    0b0111000, 0b1111111, 0b1111001 };

const char* ssid = WIFI_SSID;
const char* password = WIFI_PASSWORD;
const char* mqtt_server = MQTT_SERVER;
const char* ACKNOWLEDGE_SIGNAL = "ACKNOWLEDGE_DETECTION";
int BUTTON = 15;

WiFiClient espClient;
PubSubClient client(espClient);
unsigned long lastMsg = 0;
#define MSG_BUFFER_SIZE (50)
char msg[MSG_BUFFER_SIZE];
int value = 0;
int click = 0;
int last_click = 0;
int detected_count = 0;

void setup_wifi()
{
    delay(10);
    // We start by connecting to a WiFi network
    Serial.println();
    Serial.print("Connecting to ");
    Serial.println(ssid);

    WiFi.mode(WIFI_STA);
    WiFi.begin(ssid, password);

    while (WiFi.status() != WL_CONNECTED)
    {
        delay(500);
        Serial.print(".");
    }

    randomSeed(micros());

    Serial.println("");
    Serial.println("WiFi connected");
    Serial.println("IP address: ");
    Serial.println(WiFi.localIP());
}


void clear7SegDisplay()
{
    for (int i = 0; i < strip.numPixels(); i++)
    {
        strip.setPixelColor(i, strip.Color(0, 0, 0));
    }
}

void writeDigit(int index, int value)
{
    byte seg = segments[value];
    for (int i = 6; i >= 0; i--)
    {
        int offset = index * (PIXELS_PER_SEGMENT * 7) + i * PIXELS_PER_SEGMENT;
        uint32_t color = seg & 0x01 != 0 ? strip.Color(25, 50, 50) : strip.Color(0, 0, 0);
        for (int x = offset; x < offset + PIXELS_PER_SEGMENT; x++)
        {
            strip.setPixelColor(x, color);
        }
        seg = seg >> 1;
    }
}

void drawDefaultOledImage(void)
{
    display.clearDisplay();

    for (int16_t i = 0; i < display.height() / 2 - 2; i += 2)
    {
        display.drawRoundRect(i, i, display.width() - 2 * i, display.height() - 2 * i,
            display.height() / 4, SSD1306_WHITE);
        display.display();
        delay(1);
    }

    delay(2000);
}

void animateLoop(bool fromInner, bool clearing, uint32_t color)
{

    for (int b = fromInner ? 0 : 7; fromInner ? b < 7 : b >= 0; b += (fromInner ? 1 : -1))
    {
        if (clearing)
        {
            clear7SegDisplay();
        }
        switch (b)
        {
            case 0:
                strip.setPixelColor(19, color);
                break;
            case 1:
                strip.setPixelColor(20, color);
                strip.setPixelColor(18, color);
                break;
            case 2:
                strip.setPixelColor(0, color);
                strip.setPixelColor(8, color);
                strip.setPixelColor(9, color);
                strip.setPixelColor(17, color);
                break;
            case 3:
                strip.setPixelColor(1, color);
                strip.setPixelColor(7, color);
                strip.setPixelColor(10, color);
                strip.setPixelColor(16, color);
                break;
            case 4:
                strip.setPixelColor(2, color);
                strip.setPixelColor(6, color);
                strip.setPixelColor(11, color);
                strip.setPixelColor(15, color);
                break;
            case 5:
                strip.setPixelColor(3, color);
                strip.setPixelColor(5, color);
                strip.setPixelColor(12, color);
                strip.setPixelColor(14, color);
                break;
            case 6:
                strip.setPixelColor(4, color);
                strip.setPixelColor(13, color);
                break;
            default:
                break;
        }
        strip.show();
        delay(100);
    }
}

void read_click()
{
    click = digitalRead(BUTTON);

    if (last_click != click)
    {
        last_click = click;
        if (click == LOW)
        {
            Serial.println("Acknowledge Detection");
            char buffer[256];

            detected_count = 0;
            clear7SegDisplay();
            strip.show();
            display.clearDisplay();
            drawDefaultOledImage();

            client.publish(DOG_DETECTED_ACKNOWLEDGE_TOPIC, buffer, ACKNOWLEDGE_SIGNAL);
        }
    }
}

void callback(char* topic, byte* payload, unsigned int length)
{
    if (strcmp(topic, DOG_DETECTED_STREAM_TOPIC) == 0)
    {
        display.clearDisplay();
        display.drawBitmap(0, 0, payload, 128, 64, SSD1306_WHITE);
        display.display();
    }
    else if (strcmp(topic, DOG_DETECTED_TOPIC) == 0)
    {
        clear7SegDisplay();
        detected_count += 1;
        if (detected_count > 10)
        {
            for (int i = 0; i < 5; i++)
            {
                clear7SegDisplay();
                animateLoop(true, false, strip.Color(255, 0, 0));
                strip.show();
            }
        }
        else
        {
            for (int i = 0; i < 5; i++)
            {
                clear7SegDisplay();
                animateLoop(true, true, strip.Color(254, 221, 0));
                strip.show();
            }
            clear7SegDisplay();
            writeDigit(0, detected_count);
        }
        strip.show();
    }
    else if (strcmp(topic, DOG_DETECTED_STREAM_END_TOPIC) == 0)
    {
        Serial.println("DOG_DETECTED_STREAM_END_TOPIC");
        drawDefaultOledImage();
    }
}

void reconnect()
{
    // Loop until we're reconnected
    while (!client.connected())
    {
        Serial.print("Attempting MQTT connection...");
        // Create a random client ID
        String clientId = "ESP8266Client-";
        clientId += String(random(0xffff), HEX);
        // Attempt to connect
        if (client.connect(clientId.c_str(), MQTT_USERNAME, MQTT_PASSWORD))
        {
            Serial.println("connected");
            client.setBufferSize(16384);
            client.subscribe(DOG_DETECTED_TOPIC);
            client.subscribe(DOG_DETECTED_STREAM_TOPIC);
            client.subscribe(DOG_DETECTED_STREAM_END_TOPIC);
        }
        else
        {
            Serial.print("failed, rc=");
            Serial.print(client.state());
            Serial.println(" try again in 5 seconds");
            // Wait 5 seconds before retrying
            delay(5000);
        }
    }
}

void setup()
{
    Serial.begin(115200);
    pinMode(BUTTON, INPUT);
    // SSD1306_SWITCHCAPVCC = generate display voltage from 3.3V internally
    if (!display.begin(SSD1306_SWITCHCAPVCC, 0x3C))
    {
        Serial.println(F("SSD1306 allocation failed"));
        for (;;)
            ; // Don't proceed, loop forever
    }
    setup_wifi();
    client.setServer(mqtt_server, 1883);
    client.setCallback(callback);

    display.clearDisplay();
    drawDefaultOledImage();
    display.display();
    strip.begin();

    clear7SegDisplay();

    for (int i = 0; i < 5; i++)
    {
        animateLoop(true, true, strip.Color(144, 238, 144));
        animateLoop(false, false, strip.Color(144, 238, 144));
    }

    clear7SegDisplay();
    strip.show();
}

void loop()
{
    read_click();

    if (!client.connected())
    {
        reconnect();
    }
    client.loop();
}
