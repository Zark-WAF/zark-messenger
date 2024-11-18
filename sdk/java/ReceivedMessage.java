package io.zarkwaf.messenger;

public class ReceivedMessage {
    private final String topic;
    private final byte[] payload;

    public ReceivedMessage(String topic, byte[] payload) {
        this.topic = topic;
        this.payload = payload;
    }

    public String getTopic() {
        return topic;
    }

    public byte[] getPayload() {
        return payload;
    }
}
