package main

import (
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"os"
	"os/exec"
	"time"
)

type unixSocketDialer struct {
	addr string
}

func NewUnixSocketDialer(addr string) unixSocketDialer {
	return unixSocketDialer{addr}
}

func (u unixSocketDialer) Dial(network, addr string) (net.Conn, error) {
	return net.Dial("unix", u.addr)
}

func main() {
	rawListener, err := net.Listen("unix", "@test-client.sock")
	if err != nil {
		log.Fatal(err)
	}
	listener := rawListener.(*net.UnixListener)
	listenSocket, err := listener.File()
	// pid zuoyong, zhuyaowenw
	pid := os.Getpid()
	mntArg := fmt.Sprintf("--mnt=/proc/%d/ns/mnt", pid)
	pidArg := fmt.Sprintf("--pid=/proc/%d/ns/pid", pid)
	netArg := fmt.Sprintf("--net=/proc/%d/ns/net", pid)
	cmd := exec.Command("/usr/local/bin/nsexec", mntArg, pidArg, netArg, "--local", "--keep-fd=3", "./chaos-tproxy-demo")
	// cmd := exec.Command("/usr/local/bin/nsexec", mntArg, pidArg, netArg, "--local", "--keep-fd=3", "./server")
	cmd.ExtraFiles = []*os.File{listenSocket}
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Start()
	rawListener.Close()
	listenSocket.Close()

	dialer := NewUnixSocketDialer("@test-client.sock")
	client := http.Client{Transport: &http.Transport{Dial: dialer.Dial}}

	for {
		res, err := client.Get("http://psedo-host/")
		if err != nil {
			log.Fatal(err)
		}
		defer res.Body.Close()
		bodyBytes, err := io.ReadAll(res.Body)
		if err != nil {
			log.Fatal(err)
		}
		fmt.Printf("%s: %+v\n", time.Now(), string(bodyBytes))
		time.Sleep(time.Second)
	}

	//ctx := context.Background()
	//client, err := jrpc.DialIPC(ctx, "@test-client.sock")
	//if err != nil {
	//	log.Fatal(err, "dialing rpc client")
	//}
	//
	//for {
	//	var ret string
	//	fmt.Println("Waiting for toda to start")
	//	var rpcError error
	//	maxWaitTime := time.Millisecond * 2000
	//	timeOut, cancel := context.WithTimeout(ctx, maxWaitTime)
	//	defer cancel()
	//	rpcError = client.CallContext(timeOut, &ret, "http://psedo-host/")
	//	if rpcError != nil || ret != "ok" {
	//		fmt.Println("Starting toda takes too long or encounter an error")
	//		log.Fatal(rpcError, "toda startup takes too long or an error occurs: %s", ret)
	//	}
	//}

}
