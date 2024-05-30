package cmd

import (
	"fmt"
	"os"
	"path/filepath"
	"github.com/gemalto/flume"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var cfgFile string

var rootCmd = &cobra.Command{
	Use:   filepath.Base(os.Args[0]),
	Short: `
odido.nl aanvullers automated`,
	Run: func(cmd *cobra.Command, args []string) { 

		var log = flume.New("mypkg")
		flume.ConfigFromEnv()

		log.Info("Hello World")
		log.Error("Hello World gone wrong!")

		// fmt.Println("Hello World") 
	},
}

func Execute() {
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func init() {
	cobra.OnInitialize(initConfig)

	rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "$HOME/.config/odido-aap/config.yaml", "config file")
}

// initConfig reads in config file and ENV variables if set.
func initConfig() {
	if cfgFile == "" {
		// log.Fatalf("config file not set")
		return
	}

	viper.AutomaticEnv() // read in environment variables that match

	// If a config file is found, read it in.
	if err := viper.ReadInConfig(); err == nil {
		fmt.Fprintln(os.Stderr, "Using config file:", viper.ConfigFileUsed())
	}
	// logger.init
}
