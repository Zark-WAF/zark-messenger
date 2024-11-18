package io.zarkwaf.messenger;

import com.sun.jna.Structure;
import com.sun.jna.NativeLong;

public class IpcConfig {
    private final String sharedMemoryName;
    private final long maxMessageSize;
    private final long maxQueueSize;
    private final long maxBufferSize;

    public IpcConfig(String sharedMemoryName, long maxMessageSize, long maxQueueSize, long maxBufferSize) {
        this.sharedMemoryName = sharedMemoryName;
        this.maxMessageSize = maxMessageSize;
        this.maxQueueSize = maxQueueSize;
        this.maxBufferSize = maxBufferSize;
    }

    Structure.ByReference toNative() {
        NativeIpcConfig config = new NativeIpcConfig();
        config.sharedMemoryName = sharedMemoryName;
        config.maxMessageSize = new NativeLong(maxMessageSize);
        config.maxQueueSize = new NativeLong(maxQueueSize);
        config.maxBufferSize = new NativeLong(maxBufferSize);
        return config;
    }

    @Structure.FieldOrder({"sharedMemoryName", "maxMessageSize", "maxQueueSize", "maxBufferSize"})
    public static class NativeIpcConfig extends Structure implements Structure.ByReference {
        public String sharedMemoryName;
        public NativeLong maxMessageSize;
        public NativeLong maxQueueSize;
        public NativeLong maxBufferSize;
    }
}
