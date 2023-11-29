This is a C3D file that contains data that does not match the parameters recorded in the file
to describe the data - as a result the data will be read incorrectly.

04/19/2006  10:36 AM           302,080 sample21.c3d

The actual 3D data frame rate is 200 and the analog sample rate is 1000Hz. However the parameters 
report (incorrectly) that the POINT:RATE is 60 and the ANALOG:RATE is 1020 - both are incorrect.