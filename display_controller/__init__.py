import os
import sys

picdir = os.path.join(os.path.dirname(os.path.realpath(__file__)), "pic")
libdir = os.path.join(os.path.dirname(os.path.realpath(__file__)), "lib")
if os.path.exists(libdir):
    sys.path.append(libdir)
