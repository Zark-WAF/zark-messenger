package zark

// #cgo LDFLAGS: -lzark_waf_messenger
// #include <stdlib.h>
// #include "zark_messenger.h"
import "C"
import (
	"errors"
	"runtime"
	"sync"
	"unsafe"
)

var (
	ErrInitFailed      = errors.New("failed to initialize messenger")
	ErrSendFailed      = errors.New("failed to send message")
	ErrReceiveFailed   = errors.New("failed to receive message")
	ErrInvalidArgument = errors.New("invalid argument")
	ErrMessageTooLarge = errors.New("message too large")
	ErrNoMessages      = errors.New("no messages available")
	ErrMessengerClosed = errors.New("messenger is closed")
)

// TransportType represents the type of transport to use
type TransportType int32

const (
	TransportIPC TransportType = iota
	TransportTCP
)

// IpcConfig holds configuration for IPC transport
type IpcConfig struct {
	SharedMemoryName string
	MaxMessageSize   uint64
	MaxQueueSize     uint64
	MaxBufferSize    uint64
}

// TcpConfig holds configuration for TCP transport
type TcpConfig struct {
	Host           string
	Port           uint16
	MaxMessageSize uint64
}

// Config holds the messenger configuration
type Config struct {
	TransportType TransportType
	IpcConfig     *IpcConfig
	TcpConfig     *TcpConfig
}

// Message represents a message in the system
type Message struct {
	Topic   string
	Payload []byte
}

// Messenger represents a messenger instance
type Messenger struct {
	handle unsafe.Pointer
	mu     sync.RWMutex
	closed bool
}

// NewMessenger creates a new messenger instance with the given configuration
func NewMessenger(config *Config) (*Messenger, error) {
	var cConfig C.struct_ZarkConfig
	cConfig.transport_type = C.enum_ZarkTransportType(config.TransportType)

	// Handle IPC config
	if config.IpcConfig != nil {
		ipcConfig := config.IpcConfig
		cIpcConfig := C.struct_ZarkIpcConfig{
			shared_memory_name: C.CString(ipcConfig.SharedMemoryName),
			max_message_size:   C.size_t(ipcConfig.MaxMessageSize),
			max_queue_size:     C.size_t(ipcConfig.MaxQueueSize),
			max_buffer_size:    C.size_t(ipcConfig.MaxBufferSize),
		}
		defer C.free(unsafe.Pointer(cIpcConfig.shared_memory_name))

		cConfig.ipc_config = (*C.struct_ZarkIpcConfig)(C.malloc(C.size_t(unsafe.Sizeof(cIpcConfig))))
		defer C.free(unsafe.Pointer(cConfig.ipc_config))
		*cConfig.ipc_config = cIpcConfig
	}

	// Handle TCP config
	if config.TcpConfig != nil {
		tcpConfig := config.TcpConfig
		cTcpConfig := C.struct_ZarkTcpConfig{
			host:             C.CString(tcpConfig.Host),
			port:             C.uint16_t(tcpConfig.Port),
			max_message_size: C.size_t(tcpConfig.MaxMessageSize),
		}
		defer C.free(unsafe.Pointer(cTcpConfig.host))

		cConfig.tcp_config = (*C.struct_ZarkTcpConfig)(C.malloc(C.size_t(unsafe.Sizeof(cTcpConfig))))
		defer C.free(unsafe.Pointer(cConfig.tcp_config))
		*cConfig.tcp_config = cTcpConfig
	}

	handle := C.zark_messenger_init(&cConfig)
	if handle == nil {
		return nil, ErrInitFailed
	}

	m := &Messenger{handle: handle}
	runtime.SetFinalizer(m, (*Messenger).Close)
	return m, nil
}

// Send sends a message
func (m *Messenger) Send(msg *Message) error {
	m.mu.RLock()
	if m.closed {
		m.mu.RUnlock()
		return ErrMessengerClosed
	}
	m.mu.RUnlock()

	cTopic := C.CString(msg.Topic)
	defer C.free(unsafe.Pointer(cTopic))

	cMsg := C.struct_Message{
		topic:          cTopic,
		payload:        (*C.uchar)(unsafe.Pointer(&msg.Payload[0])),
		payload_length: C.size_t(len(msg.Payload)),
	}

	if !C.zark_messenger_send(m.handle, &cMsg) {
		return ErrSendFailed
	}
	return nil
}

// Receive receives a message
func (m *Messenger) Receive(maxTopicLen, maxPayloadLen int) (*Message, error) {
	m.mu.RLock()
	if m.closed {
		m.mu.RUnlock()
		return nil, ErrMessengerClosed
	}
	m.mu.RUnlock()

	topic := make([]byte, maxTopicLen)
	payload := make([]byte, maxPayloadLen)

	result := C.zark_messenger_receive(
		m.handle,
		(*C.char)(unsafe.Pointer(&topic[0])),
		C.size_t(maxTopicLen),
		(*C.uchar)(unsafe.Pointer(&payload[0])),
		C.size_t(maxPayloadLen),
	)

	if result < 0 {
		switch result {
		case -10:
			return nil, ErrNoMessages
		case -9:
			return nil, ErrMessageTooLarge
		default:
			return nil, ErrReceiveFailed
		}
	}

	// Find null terminator in topic
	topicLen := 0
	for i, b := range topic {
		if b == 0 {
			topicLen = i
			break
		}
	}

	return &Message{
		Topic:   string(topic[:topicLen]),
		Payload: payload[:result],
	}, nil
}

// Close closes the messenger and frees resources
func (m *Messenger) Close() error {
	m.mu.Lock()
	defer m.mu.Unlock()

	if !m.closed {
		C.zark_messenger_cleanup(m.handle)
		C.zark_messenger_free(m.handle)
		m.closed = true
	}
	return nil
}
