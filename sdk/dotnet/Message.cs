using System.Runtime.InteropServices;

namespace ZarkWaf.Messenger
{
    [StructLayout(LayoutKind.Sequential)]
    internal struct Message
    {
        [MarshalAs(UnmanagedType.LPStr)]
        public string Topic;
        [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 2)]
        public byte[] Payload;
        public UIntPtr PayloadLength;
    }
} 