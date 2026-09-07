// Sequential Go net reference for the io-model comparison matrix.
// Owned by go-check / experiments 49 and 51; retire with those comparisons. One goroutine
// owns each connection and its initialized 64 KiB buffer through ordered writes.
// There are no application-message boundaries, io.Copy/splice shortcuts, buffer
// pools, custom reactors or manually pinned goroutines in this reference.
package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"os"
	"runtime"
	"runtime/metrics"
	"strconv"
	"sync"
	"syscall"
)

const bufferBytes = 65536

func positive(text string, maximum int) (int, error) {
	n, err := strconv.Atoi(text)
	if err != nil || n <= 0 || n > maximum {
		return 0, fmt.Errorf("invalid positive integer %q (maximum %d)", text, maximum)
	}
	return n, nil
}

func echoBuffer(conn *net.TCPConn, buffer []byte) error {
	for {
		n, readErr := conn.Read(buffer)
		for offset := 0; offset < n; {
			moved, err := conn.Write(buffer[offset:n])
			offset += moved
			if err != nil {
				return err
			}
			if moved == 0 {
				return io.ErrNoProgress
			}
		}
		// Readers may return bytes and an error together. Publish those bytes
		// before observing EOF; never overwrite a partly written prefix.
		if errors.Is(readErr, io.EOF) {
			return nil
		}
		if readErr != nil {
			return readErr
		}
	}
}

// Keep the stack-owning call separate: a conditional allocation inside the
// common loop could reserve the large frame even for a supplied heap buffer.
func echoStack(conn *net.TCPConn) error {
	buffer := make([]byte, bufferBytes)
	return echoBuffer(conn, buffer)
}

func startEchoStack(conn *net.TCPConn, complete func(error)) {
	go func() { complete(echoStack(conn)) }()
}

func startEchoHeap(conn *net.TCPConn, complete func(error)) {
	// The accepting goroutine transfers this private buffer to the new handler.
	// Its asynchronous lifetime makes it heap-owned in ordinary Go; no global
	// keeper, unsafe conversion or pool is needed to force its representation.
	buffer := make([]byte, bufferBytes)
	go func() { complete(echoBuffer(conn, buffer)) }()
}

type socketState struct {
	NoDelay       int `json:"nodelay"`
	SendBuffer    int `json:"send_buffer"`
	ReceiveBuffer int `json:"receive_buffer"`
}

func socketReadback(conn *net.TCPConn, sendBuffer int) (socketState, error) {
	state := socketState{}
	raw, err := conn.SyscallConn()
	if err != nil {
		return state, err
	}
	var readErr error
	err = raw.Control(func(fd uintptr) {
		state.NoDelay, readErr = syscall.GetsockoptInt(int(fd), syscall.IPPROTO_TCP, syscall.TCP_NODELAY)
		if readErr == nil {
			state.SendBuffer, readErr = syscall.GetsockoptInt(int(fd), syscall.SOL_SOCKET, syscall.SO_SNDBUF)
		}
		if readErr == nil {
			state.ReceiveBuffer, readErr = syscall.GetsockoptInt(int(fd), syscall.SOL_SOCKET, syscall.SO_RCVBUF)
		}
	})
	if err != nil {
		return state, err
	}
	if readErr != nil {
		return state, readErr
	}
	if state.NoDelay == 0 || state.SendBuffer < sendBuffer {
		return state, fmt.Errorf("socket options did not apply: %+v", state)
	}
	return state, nil
}

func report(stage string, sockets []socketState, owner string) error {
	var memory runtime.MemStats
	runtime.ReadMemStats(&memory)
	names := []string{
		"/sched/gomaxprocs:threads", "/sched/threads/total:threads", "/sched/goroutines:goroutines",
		"/gc/gogc:percent", "/gc/gomemlimit:bytes", "/gc/cycles/total:gc-cycles",
		"/gc/heap/allocs:bytes", "/gc/heap/allocs:objects", "/gc/heap/live:bytes",
		"/memory/classes/heap/stacks:bytes", "/memory/classes/total:bytes",
		"/cpu/classes/gc/total:cpu-seconds", "/cpu/classes/total:cpu-seconds",
	}
	samples := make([]metrics.Sample, len(names))
	for i, name := range names {
		samples[i].Name = name
	}
	metrics.Read(samples)
	values := make(map[string]any, len(samples))
	for _, sample := range samples {
		switch sample.Value.Kind() {
		case metrics.KindUint64:
			values[sample.Name] = sample.Value.Uint64()
		case metrics.KindFloat64:
			values[sample.Name] = sample.Value.Float64()
		default:
			return fmt.Errorf("required runtime metric unavailable: %s", sample.Name)
		}
	}
	// Startup/exit samples are not thread or heap peaks. Runtime CPU classes
	// are runtime estimates, not substitutes for process/kernel CPU accounting.
	return json.NewEncoder(os.Stderr).Encode(map[string]any{
		"kind": "go-net", "stage": stage, "version": runtime.Version(),
		"goos": runtime.GOOS, "goarch": runtime.GOARCH, "num_cpu": runtime.NumCPU(),
		"gomaxprocs": runtime.GOMAXPROCS(0), "buffer_bytes": bufferBytes,
		"buffer_owner": owner,
		"godebug":      os.Getenv("GODEBUG"), "gogc": os.Getenv("GOGC"),
		"gomemlimit": os.Getenv("GOMEMLIMIT"), "metrics": values, "sockets": sockets,
		"memory": map[string]uint64{
			"total_alloc": memory.TotalAlloc, "heap_alloc": memory.HeapAlloc,
			"heap_inuse": memory.HeapInuse, "stack_inuse": memory.StackInuse,
			"mallocs": memory.Mallocs, "num_gc": uint64(memory.NumGC),
			"pause_total_ns": memory.PauseTotalNs,
		},
	})
}

func run() error {
	if len(os.Args) != 5 || os.Args[3] != "--threads" {
		return errors.New("usage: go_echo PORT CONNECTIONS --threads GOMAXPROCS")
	}
	port, err := positive(os.Args[1], 65535)
	if err != nil {
		return err
	}
	connections, err := positive(os.Args[2], 1<<20)
	if err != nil {
		return err
	}
	width, err := positive(os.Args[4], 1024)
	if err != nil {
		return err
	}
	runtime.GOMAXPROCS(width)
	owner := os.Getenv("WF_BENCH_GO_BUFFER_OWNER")
	if owner == "" {
		owner = "handler"
	}
	if owner != "handler" && owner != "acceptor" {
		return errors.New("WF_BENCH_GO_BUFFER_OWNER must be handler or acceptor")
	}
	observe := os.Getenv("WF_BENCH_GO_OBSERVE") == "1"
	sendBuffer := 0
	if value := os.Getenv("WF_BENCH_GO_SNDBUF"); value != "" {
		sendBuffer, err = positive(value, 1<<30)
		if err != nil {
			return err
		}
	}
	listener, err := net.ListenTCP("tcp4", &net.TCPAddr{IP: net.IPv4(127, 0, 0, 1), Port: port})
	if err != nil {
		return err
	}
	defer listener.Close()
	if observe {
		if err := report("listening", nil, owner); err != nil {
			return err
		}
	}
	var lock sync.Mutex
	active := make(map[*net.TCPConn]struct{})
	var firstError error
	var workers sync.WaitGroup
	var sockets []socketState
	fail := func(err error) {
		lock.Lock()
		defer lock.Unlock()
		if firstError == nil {
			firstError = err
			listener.Close()
			// Close interrupts outstanding Read/Write operations. Keep waiting
			// for all handlers before returning the first error to the process.
			for peer := range active {
				peer.Close()
			}
		}
	}
	for accepted := 0; accepted < connections; accepted++ {
		conn, err := listener.AcceptTCP()
		if err != nil {
			fail(err)
			break
		}
		if err = conn.SetNoDelay(true); err == nil {
			err = conn.SetKeepAlive(false)
		}
		if err == nil && sendBuffer != 0 {
			err = conn.SetWriteBuffer(sendBuffer)
		}
		if err == nil && observe {
			var state socketState
			state, err = socketReadback(conn, sendBuffer)
			sockets = append(sockets, state)
		}
		if err != nil {
			conn.Close()
			fail(err)
			break
		}
		lock.Lock()
		if firstError != nil {
			lock.Unlock()
			conn.Close()
			break
		}
		active[conn] = struct{}{}
		lock.Unlock()
		workers.Add(1)
		complete := func(err error) {
			defer workers.Done()
			closeErr := conn.Close()
			lock.Lock()
			delete(active, conn)
			lock.Unlock()
			if err != nil {
				fail(err)
			} else if closeErr != nil {
				fail(closeErr)
			}
		}
		if owner == "handler" {
			startEchoStack(conn, complete)
		} else {
			startEchoHeap(conn, complete)
		}
	}
	listener.Close()
	workers.Wait()
	if firstError != nil {
		return firstError
	}
	if observe {
		return report("complete", sockets, owner)
	}
	return nil
}

func main() {
	if err := run(); err != nil {
		fmt.Fprintln(os.Stderr, "go_echo:", err)
		os.Exit(1)
	}
}
