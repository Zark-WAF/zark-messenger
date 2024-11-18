package io.zarkwaf.messenger;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class ZarkMessengerTest {
    @Test
    public void testMessenger() {
        IpcConfig ipcConfig = new IpcConfig(
            "zark_waf_messenger_shm",
            1024,
            1024,
            1024
        );

        Config config = new Config(TransportType.IPC, ipcConfig, null);

        try (ZarkMessenger messenger = new ZarkMessenger(config)) {
            // Test sending
            String topic = "test_topic";
            byte[] payload = "Hello, World!".getBytes();
            messenger.send(topic, payload);

            // Test receiving
            ReceivedMessage received = messenger.receive(256, 1024);
            assertNotNull(received);
            assertEquals(topic, received.getTopic());
            assertArrayEquals(payload, received.getPayload());
        } catch (ZarkMessengerException e) {
            fail("Messenger operation failed: " + e.getMessage());
        }
    }
}
