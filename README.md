
# Rust File Explorer

A rust file explorer built with slint.

It is pretty slow when entering a directory with many files/folders, 
but it's because slint needs to create buttons for each file and folder 
which takes time or something similar.

Also, I can't use combo boxes in a popup window because there is an issue 
with there being multiple popup windows since combo boxes use a popup 
window so that is disappointing.

# Features

- Undo and redo buttons for traversing folders
- Refresh button to refresh the current directory
- Search button that is fairly quick when not used in the root directory
- Can enter files and edit and save the changes
- Create, delete, and rename files and folders
