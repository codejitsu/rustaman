# Rustaman

Rustaman is a command line tool for multi repository project management. The main goal of Rustaman is to enable a quick view on multiple git repositories. 

The main usage of Rustaman is to list all module directories along with ```git``` status. For instance if the main branch of your modules is ```master``` try to run the following commando to get a view on all your modules:
```
rustaman -i master 
```

<img src="https://raw.githubusercontent.com/codejitsu/rustaman/master/docs/images/rustaman-01.png" width="50%" alt="Rustaman in action"/>

## Icons

* ```master  ✔``` the module is up to date
* ```failed to open``` most likely the directory does not have a valid git reposithory within
* ```master  ✹ 1``` there are uncommited changes
* ```master  + 4``` there are unstaged files