FROM python:3.13.7

COPY ./benchmarks/requirements.txt ./requirements.txt
RUN pip install -r requirements.txt

CMD [ "jupyter", "lab", "--ip=0.0.0.0", "--no-browser", "--allow-root", "--NotebookApp.token=''", "--NotebookApp.password=''" ]