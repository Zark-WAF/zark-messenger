package io.zarkwaf.messenger;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.NativeLong;
import com.sun.jna.Structure;

public interface ZarkLib extends Library {
    ZarkLib INSTANCE = Native.load("zark_waf_messenger", ZarkLib.class);

    Pointer zark_messenger_init(Structure.ByReference config);
    boolean zark_messenger_send(Pointer messenger, Message.ByValue message);
    int zark_messenger_receive(Pointer messenger, byte[] topic, NativeLong topicLen,
                             byte[] buffer, NativeLong bufferLen);
    void zark_messenger_cleanup(Pointer messenger);
    void zark_messenger_free(Pointer messenger);
}
