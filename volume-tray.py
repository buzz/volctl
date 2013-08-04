#!/usr/bin/env python

#--------------------------------------------
# gvtray.py
# Matej Horvath <matej.horvath@gmail.com>
# 31. august 2006
#
# Let's ruin this ok?  
# http://david.chalkskeletons.com/files/volume-tray.py
# David Barr <david@chalkskeletons.com>
# 26. Jan 2008
# Changes: Remove glade dependencies, add a pretty icon,
# remove debug print things, crap like that.
#
# Poking it with a stick
# Nick Daly
# February 26, 2009
# Changes: Added a "Properties" menu which brings up the 
# specified external sound controller; correctly updates the
# volume if it's externally changed.
#
#--------------------------------------------

EXTERNAL_SOUND_CONTROLLER = "gnome-volume-control"

import sys
import os
import alsaaudio as alsa
import gtk
import egg.trayicon
import pygtk

class tray_icon:

    def __init__(self):
        self.icon = egg.trayicon.TrayIcon("voltray")
        self.volume_icon = gtk.Image()
        self.volume_icon.set_from_file("/usr/share/icons/gnome/22x22/status/audio-volume-medium.png")
        self.tray_eventbox = gtk.EventBox()  
        self.tray_eventbox.add(self.volume_icon)
        self.icon.add(self.tray_eventbox)
        self.volume = alsa.Mixer('Master').getvolume()[0]
        self.label = gtk.Label(str(self.volume))
        self.tray_eventbox.connect("button_press_event", self.clicked)
        self.tray_eventbox.connect("scroll-event", self.wheel)  
        self.icon.show_all() 
        
	# create stuffs

        self.about = about_dialog()
        self.menu = tray_menu()
        self.volume_popup = tray_volume_scale()
        self.volume_popup_exists = False
            
    #---- clicked  :the eventbox (from the trayicon) click event handler
           
    def clicked(self,widget,event):
        # update to the current system volume
        self.update()

        # right click
        if event.button == 3:
           # print "right click"
	    # open the tray_menu (includes quit and about)
            self.menu.popup(event)
	
	# left click 
        elif event.button == 1:
            #print "left click"
	    # volume_popup window is opened, close it
            if self.volume_popup_exists:
                self.volume_popup.close()
                self.volume_popup_exists = False
	    # volume_popup window is closed, open it
            else:
                self.volume_popup.create()
                self.volume_popup_exists = True
    
    #---- wheel  :the eventbox (from the trayicon) wheel scroll event handler
                  
    def wheel(self,widget,event):
	# get the current sound volume
        self.update()

	# the wheel was scrolled up - take the sound volume up
        if event.direction == gtk.gdk.SCROLL_UP:
            self.volume_up()

	# the wheel was scrolled down - take the sound volume down
        elif event.direction == gtk.gdk.SCROLL_DOWN:
            self.volume_down()           
        
	# actualize the volume
        self.update()

	# print the actual sound volume              
        #print "volume: " + str(self.volume) + "%"
        
	# if the volume_popup window is opened, actualize it
        if self.volume_popup_exists :
            self.volume_popup.scale.set_value(self.volume)
        
	# actualize the trayicon label value - new sound volume    
        self.label.set_label(str(self.volume))  
    
    #---- volume_up  :take the sound volume up in one step

    def volume_up(self):
	# if the sound is muted, unmute it and set the sound volume to 4 
        if alsa.Mixer('Master').getmute()[0] == 1:
            alsa.Mixer('Master').setmute(0) 
            alsa.Mixer('Master').setvolume(4)

	# if the volume is under 97, act normally - increase the volume for one step
        elif self.volume < 97:
            alsa.Mixer('Master').setvolume(self.volume+4)

	# the volume is over 97, set it to 100
        else:
            alsa.Mixer('Master').setvolume(100)
    
    #---- volume_down  :take the sound volume down in one step
        
    def volume_down(self):
	# if the volume is over 3, act normally - decrease it's value for one step
        if self.volume > 3:
            alsa.Mixer('Master').setvolume(self.volume-4)

	# the volume is under 3, set it to 0 and mute it
        elif self.volume > 0:
            alsa.Mixer('Master').setvolume(0)
            alsa.Mixer('Master').setmute(1)

    #---- update: actualize the volume to the system volume

    def update(self):
        self.volume = alsa.Mixer('Master').getvolume()[0]

class tray_menu:
    def __init__(self):
	# create the window object - instance of gtk.Menu
        self.window = gtk.Menu()    
	# create the menu separator menu item object
        menu_item_separator = gtk.SeparatorMenuItem()
        
	# create the about menu item object with it's icon
        menu_item_about = gtk.ImageMenuItem('gtk-about',None)
        menu_item_about.connect('activate',self.about_clicked)
        self.window.add(menu_item_about)

    # create the external sound object with icon
        menu_item_properties = gtk.ImageMenuItem('gtk-properties',None)
        menu_item_properties.connect('activate',self.properties_clicked)
        self.window.add(menu_item_properties)
        
	# add the separator menu item to the menu object	
        self.window.add(menu_item_separator)
        
	# create the quit menu item object with it's icon
        menu_item_quit = gtk.ImageMenuItem('gtk-quit',None)
        menu_item_quit.connect('activate',self.exit)
        self.window.add(menu_item_quit)
        
	# show the menu
        self.window.show_all()         
        
    def exit(self,widget):

	# quit menu item handler -> quit the application 
        gtk.main_quit()   
    
    def about_clicked(self,widget):
	# about menu item handler -> create the about window
        voltray.about.create()     
    
    def properties_clicked(self, widget):
    # properties menu item handler -> display mixer window

    # actualize the volume
        voltray.update()
        
    # call the mixer program
        import subprocess
        subprocess.call([EXTERNAL_SOUND_CONTROLLER])
        
    # actualize the volume
        voltray.update()

    # if the volume_popup window is opened, actualize it
        if voltray.volume_popup_exists :
            voltray.volume_popup.scale.set_value(voltray.volume)

    # actualize the trayicon label value - new sound volume    
        voltray.label.set_label(str(voltray.volume))
    
    def popup(self,event):
	# popup-show the menu
        self.window.popup( None, None, None, 0, event.time ); 

class about_dialog:
    def create(self):
        dialog = gtk.AboutDialog()
	dialog.set_name('Systray-Volume-Control')
	dialog.set_version('0.0.0')
	dialog.set_comments('A System Tray Volume Controller Thing')
	dialog.set_logo(gtk.gdk.pixbuf_new_from_file_at_size('/usr/share/icons/gnome/scalable/status/audio-volume-high.svg', 64, 64))
	dialog.run()
	dialog.destroy()


    def close(self):
        self.window.destroy()        
        
class tray_volume_scale:
    def create(self):
	# create the volume window 
        self.scale_window = gtk.Window(type=gtk.WINDOW_POPUP)

	# declare the window width based on the trayicon's size
	self.window_width = voltray.icon.window.get_size()[0]

	# declare the window height
        self.window_height = 120

	# get the volume window coordinates
        coordinates = self.get_tray_coordinates(voltray.icon,self.window_height,self.window_width)
        
	# place the window to the coordinates
        self.scale_window.move(coordinates[0],coordinates[1])
    
	# create the volume scale widget object
        self.scale = gtk.VScale()

	# define it's features
        self.scale.set_update_policy(gtk.UPDATE_CONTINUOUS)
        self.scale.set_digits(1)
        self.scale.set_value_pos(gtk.POS_TOP)
        self.scale.set_draw_value(False)
        self.scale.set_inverted(True)
        self.scale.set_range(0,100)

	# size the volume scale to the window's size
        self.scale.set_size_request(self.window_width,self.window_height)

	# set the actual volume scale value to the actual volume value
        self.scale.set_value(voltray.volume)

	# connect the "move" event to it's handler
        self.scale.connect("motion_notify_event", self.setted)
                
	# add the volume scale to the volume window
        self.scale_window.add(self.scale)

	# show the volume window
        self.scale_window.show_all()
        
        
    def get_tray_coordinates(self,trayicon,window_height,windows_width):
        """
        http://www.daa.com.au/pipermail/pygtk/2006-February/011837.html
        get the trayicon coordinates to send to
        notification-daemon
        trayicon=egg.trayicon.TrayIcon
        return : [x,y]
        """
        coordinates=trayicon.window.get_origin()
        size=trayicon.window.get_size()
        screen=trayicon.window.get_screen()
        screen_height=screen.get_height()

	# if the trayicon is less than 180 under the upper screen border, place the volume window under the trayicon
	# if not, place it over the trayicon
	if coordinates[1] <= window_height:
            y=coordinates[1]+size[1]
        else:
            y=coordinates[1]-window_height

        msg_xy=(coordinates[0],y)
        return msg_xy
        
    def close(self):

        self.scale_window.destroy()
        
    
    def setted(self,widget,event):
	
        voltray.volume = int(self.scale.get_value())
        alsa.Mixer('Master').setvolume(voltray.volume)

#-main----------------------------------------------------------------------------------------

if __name__ == "__main__":

    # create the tray_icon object (and icon)
    voltray = tray_icon()
    gtk.main()
