package io.zarkwaf.messenger;

import com.sun.jna.*;
import com.sun.jna.ptr.PointerByReference;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;

public class ZarkMessenger implements AutoCloseable {
    private Pointer messenger;
    private final ZarkLib lib;
    private volatile boolean closed = false;

    public ZarkMessenger(Config config) throws ZarkMessengerException {
        this.lib = ZarkLib.INSTANCE;
        
        Structure.ByReference nativeConfig = config.toNative();
        this.messenger = lib.zark_messenger_init(nativeConfig);
        
        if (this.messenger == null) {
            throw new ZarkMessengerException("Failed to initialize messenger");
        }
    }

    public synchronized void send(String topic, byte[] payload) throws ZarkMessengerException {
        checkClosed();
        
        Message.ByValue message = new Message.ByValue();
        message.topic = new Memory(topic.getBytes(StandardCharsets.UTF_8).length + 1);
        message.topic.setString(0, topic);
        
        Memory payloadMem = new Memory(payload.length);
        payloadMem.write(0, payload, 0, payload.length);
        message.payload = payloadMem;
        message.payloadLength = new NativeLong(payload.length);

        if (!lib.zark_messenger_send(messenger, message)) {
            throw new ZarkMessengerException("Failed to send message");
        }
    }

    public synchronized ReceivedMessage receive(int maxTopicLength, int maxPayloadLength) 
            throws ZarkMessengerException {
        checkClosed();
        
        byte[] topicBuffer = new byte[maxTopicLength];
        byte[] payloadBuffer = new byte[maxPayloadLength];

        int result = lib.zark_messenger_receive(
            messenger,
            topicBuffer,
            new NativeLong(maxTopicLength),
            payloadBuffer,
            new NativeLong(maxPayloadLength)
        );

        if (result < 0) {
            switch (result) {
                case -10:
                    return null; // No messages available
                case -9:
                    throw new ZarkMessengerException("Message too large");
                default:
                    throw new ZarkMessengerException("Failed to receive message");
            }
        }

        // Find null terminator in topic
        int topicLength = 0;
        for (int i = 0; i < topicBuffer.length; i++) {
            if (topicBuffer[i] == 0) {
                topicLength = i;
                break;
            }
        }

        String topic = new String(topicBuffer, 0, topicLength, StandardCharsets.UTF_8);
        byte[] payload = Arrays.copyOf(payloadBuffer, result);

        return new ReceivedMessage(topic, payload);
    }

    @Override
    public synchronized void close() {
        if (!closed) {
            lib.zark_messenger_cleanup(messenger);
            lib.zark_messenger_free(messenger);
            closed = true;
        }
    }

    private void checkClosed() throws ZarkMessengerException {
        if (closed) {
            throw new ZarkMessengerException("Messenger is closed");
        }
    }
}
