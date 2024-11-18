using System;
using System.Runtime.InteropServices;

namespace ZarkWaf.Messenger
{
    public class ZarkMessenger : IDisposable
    {
        private IntPtr _messenger;
        private bool _disposed;

        #region Native Imports
        
        [DllImport("zark_waf_messenger", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr zark_messenger_init(ref ZarkConfig config);

        [DllImport("zark_waf_messenger", CallingConvention = CallingConvention.Cdecl)]
        private static extern bool zark_messenger_send(IntPtr messenger, ref Message message);

        [DllImport("zark_waf_messenger", CallingConvention = CallingConvention.Cdecl)]
        private static extern int zark_messenger_receive(IntPtr messenger, 
            [MarshalAs(UnmanagedType.LPStr)] StringBuilder topic, 
            UIntPtr topicLen,
            [MarshalAs(UnmanagedType.LPArray)] byte[] buffer,
            UIntPtr bufferLen);

        [DllImport("zark_waf_messenger", CallingConvention = CallingConvention.Cdecl)]
        private static extern void zark_messenger_cleanup(IntPtr messenger);

        [DllImport("zark_waf_messenger", CallingConvention = CallingConvention.Cdecl)]
        private static extern void zark_messenger_free(IntPtr messenger);

        #endregion

        public ZarkMessenger(ZarkConfig config)
        {
            _messenger = zark_messenger_init(ref config);
            if (_messenger == IntPtr.Zero)
            {
                throw new ZarkMessengerException("Failed to initialize messenger");
            }
        }

        public bool Send(string topic, byte[] payload)
        {
            if (_disposed) throw new ObjectDisposedException(nameof(ZarkMessenger));

            var message = new Message
            {
                Topic = topic,
                Payload = payload,
                PayloadLength = (UIntPtr)payload.Length
            };

            return zark_messenger_send(_messenger, ref message);
        }

        public (string Topic, byte[] Payload)? Receive(int maxTopicLength = 256, int maxPayloadLength = 1024)
        {
            if (_disposed) throw new ObjectDisposedException(nameof(ZarkMessenger));

            var topic = new StringBuilder(maxTopicLength);
            var buffer = new byte[maxPayloadLength];

            int result = zark_messenger_receive(_messenger, 
                topic, 
                (UIntPtr)maxTopicLength,
                buffer, 
                (UIntPtr)maxPayloadLength);

            if (result < 0) return null;

            // Trim buffer to actual received size
            Array.Resize(ref buffer, result);
            return (topic.ToString(), buffer);
        }

        public void Dispose()
        {
            if (!_disposed)
            {
                if (_messenger != IntPtr.Zero)
                {
                    zark_messenger_cleanup(_messenger);
                    zark_messenger_free(_messenger);
                    _messenger = IntPtr.Zero;
                }
                _disposed = true;
            }
        }
    }
} 