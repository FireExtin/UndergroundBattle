import os
import sys

os.chdir('/workspace/UndergroundBattle/sites')
os.execvp('node', ['node', '--test', '--test-concurrency=1', *sys.argv[1:]])
