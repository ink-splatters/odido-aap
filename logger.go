// package main

// import (
// )
// // var logger =

// // func New(Logger *l)() (*zap.Loger) {
// func New() *zap.Logger {
//     // lumberjack.Logger is already safe for concurrent use, so we don't need to
//     // lock it.
//     w := zapcore.AddSync(&lumberjack.Logger{
//       Filename:   xdg.,
//       MaxSize:    10, // megabytes
//       MaxBackups: 5,
//       MaxAge:     28, // days
//     })

//     core := zapcore.NewCore(
//         zapcore.NewJSONEncoder(zap.NewProductionEncoderConfig()),
//         w,
//         zap.InfoLevel,
//     )
//     return zap.New(core)
// }
