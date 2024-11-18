package io.zarkwaf.messenger;

public class ZarkMessengerException extends Exception {
    public ZarkMessengerException(String message) {
        super(message);
    }

    public ZarkMessengerException(String message, Throwable cause) {
        super(message, cause);
    }
}
