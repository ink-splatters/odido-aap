package internal

import (
	"io"

	"github.com/rs/zerolog"
)

type LevelWriter struct {  
   io.Writer   
   Level       zerolog.Level  
}  
  
func (lw *LevelWriter) WriteLevel(l zerolog.Level, p []byte) (n int, err error) {  
   if l >= lw.Level {
	   return lw.Writer.Write(p)  
	}
   return len(p), nil
}


func NewLevelWriter(writer *io.Writer, zerologLevel zerolog.Level) *LevelWriter {
		return &LevelWriter{Writer: *writer, Level: zerologLevel}
}

