package logger

import (
	"os"
	"path/filepath"

	"gopkg.in/natefinch/lumberjack.v2"
)

const logFileName = "odido-aap.log"

func ensureLogFilePath() string {
	logDir := os.MkdirAll(xdg.LogDir(), os.ModePerm)
	logFilePath := filepath.Join(logDir, logFileName)
	return logFilePath
}

// TODO: config file
func NewFileWriter() *lumberjack.Logger {
	

	fileWriter := &lumberjack.Logger{  
	   Filename:   ensureLogFilePath(),  
	   MaxSize:    1,  
	   MaxAge:     30,  
	   MaxBackups: 5,  
	   LocalTime:  false,  
	   Compress:   false,  
	}  
	return fileWriter
} 