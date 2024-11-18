package io.zarkwaf.messenger;

import com.sun.jna.Structure;
import com.sun.jna.Pointer;

public class Config {
    private final TransportType transportType;
    private final IpcConfig ipcConfig;
    private final TcpConfig tcpConfig;

    public Config(TransportType transportType, IpcConfig ipcConfig, TcpConfig tcpConfig) {
        this.transportType = transportType;
        this.ipcConfig = ipcConfig;
        this.tcpConfig = tcpConfig;
    }

    Structure.ByReference toNative() {
        NativeConfig config = new NativeConfig();
        config.transportType = transportType.ordinal();
        
        if (ipcConfig != null) {
            config.ipcConfig = ipcConfig.toNative();
        }
        if (tcpConfig != null) {
            config.tcpConfig = tcpConfig.toNative();
        }
        
        return config;
    }

    @Structure.FieldOrder({"transportType", "ipcConfig", "tcpConfig"})
    public static class NativeConfig extends Structure implements Structure.ByReference {
        public int transportType;
        public Pointer ipcConfig;
        public Pointer tcpConfig;
    }
}
