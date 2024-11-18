package io.zarkwaf.messenger;

import com.sun.jna.Structure;
import com.sun.jna.NativeLong;

public class TcpConfig {
    private final String host;
    private final int port;
    private final long maxMessageSize;

    public TcpConfig(String host, int port, long maxMessageSize) {
        this.host = host;
        this.port = port;
        this.maxMessageSize = maxMessageSize;
    }

    Structure.ByReference toNative() {
        NativeTcpConfig config = new NativeTcpConfig();
        config.host = host;
        config.port = (short) port;
        config.maxMessageSize = new NativeLong(maxMessageSize);
        return config;
    }

    @Structure.FieldOrder({"host", "port", "maxMessageSize"})
    public static class NativeTcpConfig extends Structure implements Structure.ByReference {
        public String host;
        public short port;
        public NativeLong maxMessageSize;
    }
}
