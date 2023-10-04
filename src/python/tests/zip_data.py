import shutil
# Why is this so complicated in js?
shutil.make_archive('tests/zipped_testdata', format='zip', root_dir="tests/data/")
