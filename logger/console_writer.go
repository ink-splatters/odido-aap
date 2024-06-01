package logger

import (
	"os"
	"github.com/rs/zerolog"
)

func NewConsoleWriter() *zerolog.ConsoleWriter {

	consoleWriter := zerolog.NewConsoleWriter(  
		func(w *zerolog.ConsoleWriter) {
		    w.Out = os.Stderr
		},  
	)  

	return &consoleWriter
}
