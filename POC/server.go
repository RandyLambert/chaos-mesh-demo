package main

import (
	"log"
	"net"
	"net/http"
	"os"
)

/*
#include <stdio.h>
*/
import "C"

func main() {
	s := os.NewFile(3, "socket")

	listener, err := net.FileListener(s)
	if err != nil {
		log.Fatal(err)
	}

	httpServer := &http.Server{
		Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			some, err := os.ReadFile("/home/shouxunsun/GolandProjects/src/github.com/RandyLambert/chaos-mesh-demo/some")
			if err != nil {
				w.Write([]byte(err.Error()))
			} else {
				w.Write(some)
			}
		}),
	}

	err = httpServer.Serve(listener)
	if err != nil {
		log.Fatal(err)
	}
}
