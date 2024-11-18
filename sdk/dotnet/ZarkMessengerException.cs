using System;

namespace ZarkWaf.Messenger
{
    public class ZarkMessengerException : Exception
    {
        public ZarkMessengerException(string message) : base(message) { }
        public ZarkMessengerException(string message, Exception inner) : base(message, inner) { }
    }
}
