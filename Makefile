deploy:
	gcloud builds submit

image:
	sudo docker build .
