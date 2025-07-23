# PokeMMO-Companion
![CI & Release](https://img.shields.io/github/actions/workflow/status/Matzeall/PokeMMO-Companion/CI_and_Release.yml)
![Latest Release](https://img.shields.io/github/v/release/Matzeall/PokeMMO-Companion)
![License](https://img.shields.io/github/license/Matzeall/PokeMMO-Companion)
![GitHub repo size](https://img.shields.io/github/repo-size/Matzeall/PokeMMO-Companion)
![Total Downloads](https://img.shields.io/github/downloads/Matzeall/PokeMMO-Companion/total)

<img width="2556" height="1402" alt="image" src="https://github.com/user-attachments/assets/ddf3d663-f058-4389-a5ab-6953226d52a4" />


## Contents
* [Installation](#installation)
* [Features](#features)
    * [In-Game Overlay](#overlay-functionality)
    * [Notes / ToDo's](#notes-atl--n)
    * [Resources](#resources-alt--r)
    * [Type Matrix](#type-matrix-atl--t)
* [Example Usage](#example-usage)
* [Contributing and Support](#contributing-and-support)
* [License](#license)

## Installation
1. Download the latest release from [releases section](https://github.com/Matzeall/PokeMMO-Companion/releases)
2. Extract the application archive to where you typically store your applications (location doesn't matter, aslong the contents of the PokeMMO-Companion folder stay on the same relative path to one another)
3. Open the PokeMMO-Companion executable (PokeMMO-Companion.exe on Windows, PokeMMO-Companion on Linux)

> [!NOTE]
> You must also repeat this process to update to a newer release, since I haven't had the time to make a proper installer.\
> Just be sure to copy over all custom changes you made to the resources folder.

> [!WARNING]
> The Ubuntu(Linux) release doesn't yet work well as an overlay over PokeMMO. For now disabling the overlay functionality in the settings (Alt + O) and switching between PokeMMO and the companion-app by Alt-tabbing is probably the most convenient way to use the app on ubuntu.\
> I experimented with some stuff for wayland compositors, but it turns out to be quite clunky right now.
> To get it somewhat working you could accept the sudo privileges for noticing hotkeys at application start and then manually click on the top application bar -> select "Always on top", since e.g. Gnome DE sadly ignores all automated calls to set the layer. Maximizing the window also negates the "Always on top"-effect, therefore I prohibit it.\
> But since I mainly play on ubuntu I will have a look into the more relaxed X11 frontend soon, and try to get the overlay functionality working there too. Please open an issue if you also wait for that support, so I know there is a demand for it. 
 
## Features
#### Overlay Functionality
- switch between 3 modes (Focused, Visible, Closed) with Alt + F(ocused) / V(isible) / C(losed)
    - **Focused** (Alt + F):
        - clicks & hotkeys interact with the overlay now
        - enables action-bar in the bottom center which can be used to open the feature windows 
        <img width="359" height="85" alt="image" src="https://github.com/user-attachments/assets/95107ce9-f634-4233-aa82-e77a7ca23056" />\
    - **Visible** (Alt + V):
        - the overlay is not interactable, but still visible on-top of PokeMMO
     
    - **Closed** (Alt +C):
        - the overlay is closed/hidden fully 

- overlay can also be disabled entirely, to have it behave like any other opaque window, if you want to put it on a second monitor anyway.
  
---

#### Notes (Atl + N)
- persistent Notes/ToDo system with simple styling options, so you can easily keep track of your plans and progress
  
<img width="585" height="295" alt="Notes" src="https://github.com/user-attachments/assets/b6feb5f2-96ee-4db8-89b9-9bc8e54bb357" />\

---

#### Resources (Alt + R)
- see all resources you need directly in the overlay instead of searching the forums and such everytime again
  
<img width="555" height="494" alt="Resources" src="https://github.com/user-attachments/assets/ddd09dab-bae2-4577-9146-3d0dd2977c7e" />\

- easily write your own resources, if you need something else often
  - just add or edit the md-files in the "assets/resources/" folder
  - they support **plain text** (markdown format: headings, bold text, tables), **AppLinks** (links to other md-files) and **WebLinks** (open in browser when clicked)
    
    <img width="623" height="159" alt="image" src="https://github.com/user-attachments/assets/ac71fe32-94b6-4c0e-b46a-0bc1d7f86b8e" />\
    ![WriteCustomResources_Guide](https://github.com/user-attachments/assets/11fec385-c5d6-49f5-bb99-5c233e37f32f)\

---

#### Type Matrix (Atl + T)
- open a scrollable Type-Matrix to quickly remind you what's effective against poison types
- the window can be resized and positioned wherever it suits your screen layout and acts as a helpful reminder in e.g. PVP battles
  
  <img width="367" height="374" alt="image" src="https://github.com/user-attachments/assets/78e71db7-aa62-43c4-99b2-125f39147eda" />\
  
---
  
## Example Usage

#### Notes & Resources

https://github.com/user-attachments/assets/6fe6287b-697f-4637-8af0-9f048c784b6a


## Contributing and Support
For all issues, feature requests or support questions please open a new [issue](https://github.com/Matzeall/PokeMMO-Companion/issues). I am always happy to help.\
All pull requests are welcome, but they must be approved by me before being merged.

## License
This project is under the standard [MIT License](https://github.com/Matzeall/PokeMMO-Companion/blob/main/LICENSE).\
Do anything you like with it, but keep it free for everyone.
