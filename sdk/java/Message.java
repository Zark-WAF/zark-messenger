package io.zarkwaf.messenger;

import com.sun.jna.Structure;
import com.sun.jna.Pointer;
import com.sun.jna.NativeLong;

@Structure.FieldOrder({"topic", "payload", "payloadLength"})
public class Message extends Structure {
    public Pointer topic;
    public Pointer payload;
    public NativeLong payloadLength;

    public static class ByValue extends Message implements Structure.ByValue {}
}
