.PHONY:
	clean
build:
	go build -o aap main.go

run:
	go run main.go


clean:
	rm -f ./aap ./odido-aap
