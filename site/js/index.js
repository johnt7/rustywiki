'use strict';

var remoteLogArray = [];
var remoteLogRequestInProcess = false;

var myLockString = "";
var latestVersionName = "current";
var configuration = {};
var imageIndex = { path: "", dir:[]};
var currentImageIndex = [];
var remoteLogEnabled = true;

document.onmouseover = function() {
    window.innerDocClick = true;
}

document.onmouseleave = function() {
    window.innerDocClick = false;
}
window.onhashchange = function() {
	debugLog( "HASH Change: " + window.innerDocClick + " " + window.location.hash);
}
var transformPatterns = [
	[ /\n(\#\#\#\#\#\#)([^\n]*)(?=\n)/g, '<ooooooli>$2</ooooooli>' ], // \n######
	[ /(<ooooooli>[^\n]*<\/ooooooli>)/g, '<ol class="olLevel6">$1</ol>'],

	[ /\n(\*\*\*\*\*\*)([^\n]*)(?=\n)/g, '<uuuuuuli>$2</uuuuuuli>' ], // \n*****
	[ /(<uuuuuuli>[^\n]*<\/uuuuuuli>)/g, '<ul class="ulLevel6">$1</ul>'],

	[ /\n(\#\#\#\#\#)([^\n]*)(?=\n)/g, '<oooooli>$2</oooooli>' ], // \n#####
	[ /(<oooooli>[^\n]*<\/oooooli>)/g, '<ol class="olLevel5">$1</ol>'],

	[ /\n(\*\*\*\*\*)([^\n]*)(?=\n)/g, '<uuuuuli>$2</uuuuuli>' ], // \n*****
	[ /(<uuuuuli>[^\n]*<\/uuuuuli>)/g, '<ul class="ulLevel5">$1</ul>'],

	[ /\n(\#\#\#\#)([^\n]*)(?=\n)/g, '<ooooli>$2</ooooli>' ], // \n####
	[ /(<ooooli>[^\n]*<\/ooooli>)/g, '<ol class="olLevel4">$1</ol>'],

	[ /\n(\*\*\*\*)([^\n]*)(?=\n)/g, '<uuuuli>$2</uuuuli>' ], // \n****
	[ /(<uuuuli>[^\n]*<\/uuuuli>)/g, '<ul class="ulLevel4">$1</ul>'],

	[ /\n(\#\#\#)([^\n]*)(?=\n)/g, '<oooli>$2</oooli>' ], // \n###
	[ /(<oooli>[^\n]*<\/oooli>)/g, '<ol class="olLevel3">$1</ol>'],

	[ /\n(\*\*\*)([^\n]*)(?=\n)/g, '<uuuli>$2</uuuli>' ], // \n***
	[ /(<uuuli>[^\n]*<\/uuuli>)/g, '<ul class="ulLevel3">$1</ul>'],

	[ /\n(\#\#)([^\n]*)(?=\n)/g, '<ooli>$2</ooli>' ], // \n##
	[ /(<ooli>[^\n]*<\/ooli>)/g, '<ol class="olLevel2">$1</ol>'],

	[ /\n(\*\*)([^\n]*)(?=\n)/g, '<uuli>$2</uuli>' ], // \n**
	[ /(<uuli>[^\n]*<\/uuli>)/g, '<ul class="ulLevel2">$1</ul>'],

	[ /\n(\#)([^\n]*)(?=\n)/g, '<oli>$2</oli>' ], // \n#
	[ /(<oli>[^\n]*<\/oli>)/g, '<ol class="olLevel1">$1</ol>'],

	[ /\n(\*)([^\n]*)(?=\n)/g, '<uli>$2</uli>' ], // \n*
	[ /(<uli>[^\n]*<\/uli>)/g, '<ul class="ulLevel1">$1</ul>'],

	[ /<u+li>/g, '<li class="ulListItem">' ], // 
	[ /<\/u+li>/g, '</li>' ], // 
	[ /<o+li>/g, '<li class="olListItem">' ], // 
	[ /<\/o+li>/g, '</li>' ], // 

	[ /\\\+\+\+/g, '<THISISOPENINGTRIPLEPLUSES>' ], // \+++
	[ /\\<</g, '<THISISOPENINGDOUBLELESSTHANS>' ], // \<<
	[ /\\\[\[/g, '<THISISOPENINGDOUBLESQUAREBRACKETS>' ], // \[[
	[ /\\\[/g, '<THISISOPENINGSQUAREBRACKETS>' ], // \[

	[ /<!--[ \t]*WIKI[ \t]*HEADER[ \t]*LEVEL[ \t]*([0123456])[ \t]*-->/g, "<!--WIKIHEADERLEVEL $1-->"], 

	[ /\n=======[ |\t]*([^= \t]*)[ |\t]*=======/g, '<!--WIKISECTION--><h6 id="$1">$1</h6>'], // h6
	[ /\n======[ |\t]*([^= \t]*)[ |\t]*======/g, '<!--WIKISECTION--><h5 id="$1">$1</h5>'], // h5
	[ /\n=====[ |\t]*([^= \t]*)[ |\t]*=====/g, '<!--WIKISECTION--><h4 id="$1">$1</h4>'], // h4
	[ /\n====[ |\t]*([^= \t]*)[ |\t]*====/g, '<!--WIKISECTION--><h3 id="$1">$1</h3>'], // h3
	[ /\n===[ |\t]*([^= \t]*)[ |\t]*===/g, '<!--WIKISECTION--><h2 id="$1">$1</h2>'], // h2
	[ /\n==[ |\t]*([^= \t]*)[ |\t]*==/g, '<!--WIKISECTION--><h1 id="$1">$1</h1>'], // h1

	[ /\[\[\>([^\]\|]*)\]\]/g, '<video controls muted width="50%"><source src="/media/$1"></video>' ], // video
	[ /\[\[\>([^\]\|]*)\|([\d]*)\]\]/g, '<video controls muted width="$2%"><source src="/media/$1"></video>' ], // video width
	[ /\[\[\|([^\]\|]*)\]\]/g, '<img src="/media/$1" title="$1" width="50%">' ], // image
	[ /\[\[\|([^\]\|]*)\|([^\]\|]*)\]\]/g, '<img src="/media/$1" title="$2" width="50%">' ], // image title
	[ /\[\[\|([^\]\|]*)\|([^\]\|]*)\|([\d]*)\]\]/g, '<img src="/media/$1" title="$2" width="$3%">' ], //  image title width%
	[ /\[\[\|([^\]\|]*)\|([^\]\|]*)\|([\d]*)\|([^\]\|]*)\]\]/g, '<img style="$4" src="/media/$1" title="$2" width="$3%">' ],  // image title width% css transform

	[ /\+\+\+(\d)/g, '<hr style="border-width:$1px;">'], // +++n
	[ /\+\+\+/g, '<hr style="border-width:1px";>'], // +++

	[ /\[\[([^\]\|]*)\]\]/g, '<span class="localLink" onclick="localLinkOnClick(\'$1\')">$1</span>' ], // locallink
	[ /\[\[([^\]\|]*)\|([^\]\|]*)\]\]/g, '<span class="localLink" onclick="localLinkOnClick(\'$1\')">$2</span>' ], // locallink name
	[ /\n\n\n/g, '<br><br>'], // cr cr
	[ /\n\n/g, '<br>'], // cr
	[ /<</g, '<br>'], // cr
	[ /<br>\n<br>/g, '<br><br><br>'], // cr
	[ /<THISISOPENINGTRIPLEPLUSES>/g, '+++' ], // \[[
	[ /<THISISOPENINGDOUBLELESSTHANS>/g, '<<' ], // \<<
	[ /<THISISOPENINGDOUBLESQUAREBRACKETS>/g, '\[\[' ], // \[[
	[ /<THISISOPENINGSQUAREBRACKETS>/g, '\[' ], // \[
];

var stringTransformPatterns = [
	[ /\\/g,'\\\\' ],
	[ /\n/g,'\\n' ],
	[ /\\/g,'\\\\' ],
	[ /\"/g,'\\"' ]
];

function addHeaders( page) {
	try {
		var i, pos, headerLevel = 9, mp = '<!--WIKISECTION-->', ml = "<!--WIKIHEADERLEVEL ", header = '<div class="sectionHeaderDiv">', level = 0, currentId = [], cid;
		pos = page.indexOf( ml);
		if( pos > 0) {
			pos += ml.length;
			var sl = page.slice(pos, pos+1);
			var headerLevel = parseInt( sl, 10);
		}
		for( pos = page.indexOf( mp); pos >= 0; pos = page.indexOf( mp, pos)) {
			pos += mp.length + 2;
			var sl = page.slice(pos, pos+1);
			var newLevel = parseInt( sl, 10);
			pos += 6;
			if( newLevel <= headerLevel) {
				var qi = page.indexOf('"', pos);
				var h = 'x';
				for( i = level; i < newLevel; i++) {
					currentId.push( h);
					header += '<ol class="sectionHeader">\n';
				}
				for( i = level; i > newLevel; i--) {
					currentId.pop( );
					header += '</ol>\n';
				}
				if( qi >= 0) {
					h = page.slice(pos, qi);
				}
				currentId.pop( );
				currentId.push( h);
				cid = currentId.join('_');
				header += '<li class="sectionLink" onclick="location.hash = \'';
				var b = page.slice( 0, pos);
				var e = page.slice( qi);
				page =  b + cid + e;
				header += cid + '\';">' + h + '</li>\n';
				pos +=  h.length;
				level = newLevel;
			}
		}
		for( ; level > 0; level--) {
			currentId.pop( );
			header += '</ol>\n';
		}
		header += "</div>"
		return header + page;

	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function applyTransforms( p) {
	try {
		var i;
		for( i = 0; i < transformPatterns.length; i++) {
			p = p.replace(transformPatterns[i][0],transformPatterns[i][1]);
		}
		return p;
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function bodyonload() {
	try {
		debugLog( 'bodyonload');
		myLockString = "helosLock" + Math.random();
		document.getElementById( 'ShowButton').disabled = true;
		document.getElementById( 'EditButton').disabled = true;
		document.getElementById( 'CancelButton').disabled = true;
		document.getElementById( 'SaveButton').disabled = true;
		document.getElementById( 'BackImagesButton').disabled = true;
		document.getElementById('logoutDiv').style.display = '';
		document.getElementById('UpdateCommentArea').style.display = 'none';
		document.getElementById('ShowChangePasswordDiv').style.display = 'none';
		document.getElementById('ShowDiv').style.display = 'none';
		document.getElementById('ShowImagesDiv').style.display = 'none';
		document.getElementById('ShowUploadDiv').style.display = 'none';
		document.getElementById('EditArea').style.display = 'none';
		document.getElementById('DisplayArea').style.display = '';
		loadConfigPage( );
		debugLog( 'bodyonload DONE');
		//throw new Error( 'an execption');
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function cleanResponse( r) {
	try {
		return r.replace(/\r\n/g,'\n');
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function configLoaded( page, version, req) {
	try {
		if( req.status == 200 ) {
			debugLog( 'configLoaded: ' + page + ':' + version + '::' + req.status);
			var c = parsePage( cleanResponse(req.responseText));
			configuration = JSON.parse( c[1] );
			if( STARTPAGE && STARTPAGE != "" && STARTPAGE != "DUMMYSTARTPAGE") {
				loadWikiPage( STARTPAGE, latestVersionName);				
			} else if( configuration.StartPage && configuration.StartPage != "") {
				loadWikiPage( configuration.StartPage, latestVersionName);				
			} else {
				// no page specified
			}
		} else {
			configLoadFailed( page, version, req);
		}
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function configLoadFailed( page, version, req) {
	try {
		errorLog( 'configLoadFailed: ' + + page + ':' + version + '::' + req.status);
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function debugLog( t) {
	remoteLog( '/jsLog/Debug', t);
}
function errorLog( t) {
	remoteLog( '/jsLog/Error', t);
	if( document.getElementById('ErrorDisplay').innerHTML == "<br>") {
		document.getElementById('ErrorDisplay').innerHTML = t ;
	} else {
		document.getElementById('ErrorDisplay').innerHTML += "<br>" + t;
	}
	setTimeout( function(){ document.getElementById('ErrorDisplay').innerHTML = "<br>" }, 5000);
}
function imageClick( img, event, id, t) {
	try {
		debugLog( 'imageClick: '+ id + ' ' + t);
		EXIF.enableXmp();
		EXIF.getData( img, function() {
			var make = EXIF.getTag(img, "Make");
		});
		// var img = document.getElementById(id );
	  	img.style.visibility = 'hidden';
		setTimeout( function(){ img.style.visibility = 'visible'; }, 300);
		var tx = document.createElement('textarea');
  		tx.value = t;
  		document.body.appendChild( tx);
  		tx.select();
  		document.execCommand('copy');
  		document.body.removeChild(tx);
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function imageIndexLoaded( req) {
	try {
		debugLog( 'imageIndexLoaded');
		var o  = JSON.parse( cleanResponse(req.responseText));
		if( !o || !o.dir || o.dir.length <= 0) {
			imageIndex = { path: "", dir:[]};
		} else {
			imageIndex = o;
		}
		currentImageIndex = [ o ];
		updateImages();
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function javascriptExceptionLog( e) {
	remoteLog( '/jsLog/Exception',  e.stack);
}
function loadConfigPage() {
	try {
		loadPage( '_config', latestVersionName, configLoaded, configLoadFailed);
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function loadPage( page, version, fsuccess, ffail) {
	try {
		var req = new XMLHttpRequest();
		var currentPage = page + '/' + version;
		req.onload = function() {
			setTimeout( function(){ fsuccess(page, version, req);}, 10);
		};
		req.timeout = 3000;
		req.ontimeout =  function() {
			setTimeout( function(){ ffail(page, version, req);}, 10);
		};
		req.open('GET', '/wiki/'+ currentPage, true);
		req.send( );
	} catch( e) {
		javascriptExceptionLog( e);
	}	
}
function loadWikiPage( page, version) {
	try {
		loadPage( page, version, wikiPageLoaded, wikiPageLoadFailed);
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function localLinkCreate( page) {
	try {
		debugLog( 'localLinkCreate: ' + page);
		var h = '{\n\t"Page": "'+ page+ '",\n\t"Revision": "000000000",\n\t"PreviousRevision": "000000000",\n\t"CreateDate": "' +
			timeStamp() + '",\n\t"RevisionDate": "' + timeStamp() + '",\n\t"RevisedBy": "sal",\n\t"Comment": "Initial save"\n}\n';
		document.getElementById('PageName').innerText = page;
		document.getElementById('ShowArea').innerText = h;
		document.getElementById('EditArea').value = "";
		document.getElementById('DisplayArea').innerHTML = transform( "");
		document.getElementById( 'ShowButton').disabled = false;
		document.getElementById( 'EditButton').disabled = false;
		setTimeout( function(){ onEditButton( );}, 10);

	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function localLinkLoaded( page, version, req) {
	try {
		if( req.status == 200 ) {
			document.getElementById('PageName').innerText = page;
			var d = parsePage(cleanResponse(req.responseText));
			document.getElementById('ShowArea').innerText = d[0];
			document.getElementById('EditArea').value = d[1];
			document.getElementById('DisplayArea').innerHTML = transform( d[1]);
			document.getElementById( 'ShowButton').disabled = false;
			document.getElementById( 'EditButton').disabled = false;
			debugLog( 'localLinkLoaded: ' + page + ':' + version + '::' + req.status);
		} else if( req.status == 404 ) {
			localLinkCreate( page);
		} else {
			localLinkLoadFailed( page,version, req);
		}
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function localLinkLoadFailed( page, version, req) {
	try {
		errorLog( 'localLinkLoadFailed: ' + currentPage + '::' + req.status);
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function localLinkOnClick( page) {
	try {
		debugLog( 'localLinkOnClick: ' + page);
		loadPage( page, latestVersionName, localLinkLoaded, localLinkLoadFailed);

	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function lockWikiPage( lsuccess, lfail) {
	try {
		var req = new XMLHttpRequest();
		var d = {
			Page: document.getElementById('PageName').innerText,
			Lock: myLockString
		};

		req.onload = function() {
			setTimeout( function(){ lsuccess( d.Page, req);}, 10);
		};
		req.timeout = 3000;
		req.ontimeout =  function() {
			setTimeout( function(){ lfail( d.Page, req);}, 10);
		};
		req.open('POST', '/jsUser/Wikilock', true);
		req.send( JSON.stringify( d));
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onBackImagesButton() {
	try {
		debugLog( 'onBackImagesButton');
		currentImageIndex.pop();
		document.getElementById( 'BackImagesButton').disabled = currentImageIndex.length <= 1;
		updateImages();
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onCancelButton() {
	try {
		debugLog( 'onCancelButton');
		unlockWikiPage( onCancelUnlockSuccess, onCancelUnlockFailed);
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function cancelEditMode( ) {
	try {
		debugLog( 'cancelEditMode');
		document.getElementById('EditArea').style.display = 'none';
		document.getElementById('DisplayArea').style.display = '';
		document.getElementById('UpdateCommentArea').style.display = 'none';
		document.getElementById( 'EditButton').disabled = false;
		document.getElementById( 'CancelButton').disabled = true;
		document.getElementById( 'SaveButton').disabled = true;
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onCancelUnlockFailed( page, req) {
	try {
		debugLog( 'onCancelUnlockFailed');
		cancelEditMode();
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onCancelUnlockSuccess( page, req) {
	try {
		debugLog( 'onCancelUnlockSuccess');
		if( req.status == 200 ) {
			cancelEditMode();
		} else {
			onCancelUnlockFailed( page, req);
		}
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onDirClick( i) {
	try {
		debugLog( 'onDirClick: ' + i);
		var d = currentImageIndex[ currentImageIndex.length - 1];
		var nd = d.dir[ i].dir;
		currentImageIndex.push( nd);
		document.getElementById( 'BackImagesButton').disabled = false;
		updateImages();
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onEditButton() {
	try {
		debugLog( 'onEditButton');
		lockWikiPage( onEditLockSuccess, onEditLockFailed);
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onEditLockFailed( page, req) {
	try {
		debugLog( 'onEditLockFailed');
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onEditLockSuccess( page, req) {
	try {
		debugLog( 'onEditLockSuccess');
		if( req.status == 200 ) {
			document.getElementById( 'EditArea').style.display = '';
			document.getElementById( 'DisplayArea').style.display = 'none';
			document.getElementById( 'UpdateCommentArea').style.display = '';
			document.getElementById( 'EditButton').disabled = true;
			document.getElementById( 'CancelButton').disabled = false;
			document.getElementById( 'SaveButton').disabled = false;
		} else {
			onEditLockFailed( page, req);
		}
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onLogout( ) {
	try {
		debugLog( 'onLogout');
		document.getElementById('logoutDiv').style.display = 'none';
		var req = new XMLHttpRequest();
		req.onload = function() {
			console.log( 'Logout success');
		};
		req.timeout = 3000;
		req.ontimeout =  function() {
			console.log( 'Logout failure');
		};
		req.open('GET', '/page/start', true);
		req.setRequestHeader("Authorization", "Basic " + btoa("username:password"));
		req.send( );
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onMasterResetButton( ) {
	try {
		var req = new XMLHttpRequest();
		req.onload = function() {
			setTimeout( function(){ }, 10);
		};
		req.timeout = 1000;
		req.ontimeout =  function() {
			setTimeout( function(){ errorLog( "MasterReset failed.");}, 10);
		};
		req.open('GET', '/jsAdmin/MasterReset', true);
		req.send( );
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onChangePassword( ) {
	try {
		debugLog( 'onChangePassword');
		if( document.getElementById('ShowChangePasswordDiv').style.display == '') {
			document.getElementById('ShowChangePasswordDiv').style.display = 'none';
			document.getElementById('ChangePasswordButton').innerText = 'Show Change Password';
		} else {
			document.getElementById('ShowChangePasswordDiv').style.display = '';
			document.getElementById('ChangePasswordButton').innerText = 'Hide Change Password';
			refreshImages();
		}
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onSubmitUserDelete() {
	try {
		debugLog( 'onSubmitUserDelete()');
		var req = new XMLHttpRequest();
		var d = {
			User: document.getElementById('PWUser').value,
		};
		req.onload = function() {
			if( req.status == 200 ) {
				debugLog( 'User delete success');
			} else {
				errorLog( 'User delete failure');
			}
		};
		req.timeout = 3000;
		req.ontimeout =  function() {
			errorLog( 'User delete failure');
		};
		req.open('POST', '/jsAdmin/UserDelete', true);
		req.send( JSON.stringify( d));
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onSubmitUserModify() {
	try {
		debugLog( 'onSubmitUserModify()');
		var req = new XMLHttpRequest();
		var d = {
			User: document.getElementById('PWUser').value,
			Password: document.getElementById('PWPassword').value,
			NewPassword: document.getElementById('PWNewPassword').value,
			NewPasswordCheck: document.getElementById('PWNewPasswordCheck').value,
			Comment: document.getElementById('PWComment').value,
		};
		req.onload = function() {
			if( req.status == 200 ) {
				debugLog( 'User modify success');
			} else {
				errorLog( 'User modify failure');
			}
		};
		req.timeout = 3000;
		req.ontimeout =  function() {
			errorLog( 'User modify failure');
		};
		req.open('POST', '/jsUser/UserModify', true);
		req.send( JSON.stringify( d));
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onSaveButton() {
	try {
		debugLog( 'onSaveButton');
		var d = document.getElementById('EditArea').value;
		var c = toSafeJSONString( document.getElementById('UpdateComment').value);
		var ht = document.getElementById('ShowArea').innerText;
		var h = JSON.parse( ht);
		h.PreviousRevision = h.Revision;
		var r = parseInt(h.Revision) + 1;
		h.Revision = ("000000000" + r).substr(-9,9);
		h.Comment = c;
		h.Lock = myLockString;
		h.Data = d;
		saveWikiPage(h,  onSaveSuccess, onSaveFailure);		
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onSaveSuccess( h, req) {
	try {
		if( req.status == 200) {
			debugLog( 'onSaveSuccess: ' + h.Page + '::' + req.status);
			unlockWikiPage( onSaveUnlockSuccess, onSaveUnlockFailed);
		} else {
			onSaveFailure( pd, req);
		}
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onSaveFailure( h, req) {
	try {
		errorLog( 'onSaveFailure: ' + pd.Page + '::' + pd.Version + '::' + req.status);
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onSaveUnlockFailed( page, req) {
	try {
		debugLog( 'onSaveUnlockFailed');
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onSaveUnlockSuccess( page, req) {
	try {
		debugLog( 'onSaveUnlockSuccess');
		if( req.status == 200 ) {
			document.getElementById('EditArea').style.display = 'none';
			document.getElementById('DisplayArea').style.display = '';
			document.getElementById('UpdateCommentArea').style.display = 'none';
			document.getElementById( 'EditButton').disabled = false;
			document.getElementById( 'CancelButton').disabled = true;
			document.getElementById( 'SaveButton').disabled = true;
			loadWikiPage( page, latestVersionName);				

		} else {
			onSaveUnlockFailed( page, req);
		}
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onShowButton() {
	try {
		debugLog( 'onShowButton');
		if( document.getElementById('ShowDiv').style.display == '') {
			document.getElementById('ShowDiv').style.display = 'none';
			document.getElementById('ShowButton').innerText = 'Show Metadata';
		} else {
			document.getElementById('ShowDiv').style.display = '';
			document.getElementById('ShowButton').innerText = 'Hide Metadata';
		}
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onShowImagesButton() {
	try {
		debugLog( 'onShowImagesButton');
		if( document.getElementById('ShowImagesDiv').style.display == '') {
			document.getElementById('ShowImagesDiv').style.display = 'none';
			document.getElementById('ShowImagesButton').innerText = 'Show Images';
		} else {
			document.getElementById('ShowImagesDiv').style.display = '';
			document.getElementById('ShowImagesButton').innerText = 'Hide Images';
			refreshImages();
		}
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onShowUploadButton() {
	try {
		debugLog( 'onShowUploadButton');
		if( document.getElementById('ShowUploadDiv').style.display == '') {
			document.getElementById('ShowUploadDiv').style.display = 'none';
			document.getElementById('ShowUploadButton').innerText = 'Show Upload Area';
		} else {
			document.getElementById('ShowUploadDiv').style.display = '';
			document.getElementById('ShowUploadButton').innerText = 'Hide Upload Area';
			refreshImages();
		}
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onTextChange( ) {
	try {
		debugLog( 'onTextChange');
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function onUpdatePreviewButton() {
	try {
		debugLog( 'onUpdatePreviewButton');
		document.getElementById('DisplayArea').innerHTML = transform( document.getElementById('EditArea').value);
		document.getElementById('DisplayArea').style.display = '';

	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function parsePage( page) {
	try {
		return page.split('\n<!--REVISION HEADER DEMARCATION>\n');
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function remoteLog( u, m) {
	if( remoteLogEnabled) {
		remoteLogArray.push( u);
		var d = {
			LogText: m
		};
		remoteLogArray.push( JSON.stringify( d));
		remoteLogSend();
	}
}
function remoteLogSend( ) {
	if( remoteLogArray.length > 0 && remoteLogRequestInProcess == false) {
		remoteLogRequestInProcess = true;
		var url = remoteLogArray.shift();
		var message = remoteLogArray.shift();
		var req = new XMLHttpRequest();
		req.onload = function() {
			if( req.status == 510) {
				remoteLogArray = [];
			}
			remoteLogRequestInProcess = false;
			setTimeout( function(){ remoteLogSend();}, 10);
		};
		req.timeout = 3000;
		req.ontimeout =  function() {
			console.log('ERROR: remoteLogSend failed.');
			remoteLogRequestInProcess = false;
			setTimeout( function(){ remoteLogSend();}, 10);
		};
		req.open('POST', url, true);
		req.send( message);
	}
}
function refreshImages() {
	try {
		debugLog( 'refreshImages');
		var req = new XMLHttpRequest();
		req.onload = function() {
			setTimeout( function(){ imageIndexLoaded( req);}, 10);
		};
		req.timeout = 3000;
		req.ontimeout =  function() {
			setTimeout( function(){ throw new Error( 'Failed to load image index');}, 10);
		};
		req.open('GET', '/page/MediaIndex', true);
		req.send( );
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function saveWikiPage(h, ssuccess, sfail) {
	try {
		var ds, req = new XMLHttpRequest();
		req.onload = function() {
			setTimeout( function(){ ssuccess( h, req);}, 10);
		};
		req.timeout = 30000;
		req.ontimeout =  function() {
				setTimeout( function(){ sfail(h, req);}, 10);
		};
		req.open('POST', '/jsUser/Wikisave', true);
		ds = JSON.stringify(h);
		req.send( ds);
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function timeStamp() {
	var __jsv__d = new Date();
	var tz = __jsv__d.getTimezoneOffset();
	var ts = "+";

	if( tz < 0) {
		ts = "-";
		tz = 0 - tz;
	}
	var tzh = Math.floor(tz/60);
	var tzm = Math.round(tz - (tzh*60), 0);
	ts = ts + ("0" + tzh.toString()).substr(-2,2) + ":" + ("0" + tzm.toString()).substr(-2,2)
	return __jsv__d.getFullYear().toString() + "/" +
	("0" +(__jsv__d.getMonth() + 1)).substr(-2,2) + "/" +
	("0" +__jsv__d.getDate()).substr(-2,2) + " " +
	("0" +__jsv__d.getHours()).substr(-2,2) + ":" +
	("0" +__jsv__d.getMinutes()).substr(-2,2) + ":" +
	("0" +__jsv__d.getSeconds()).substr(-2,2) + "." +
	("00" +__jsv__d.getMilliseconds()).substr(-3,3) + ts;
}
function toSafeJSONString( s) {
	var i;
	for( i = 0; i < stringTransformPatterns.length; i++) {
		s = s.replace( stringTransformPatterns[i][0], stringTransformPatterns[i][1]);
	}
	return s;
}
function transform( page) {
	try {
		var i, j, pres, comments;
		page = '\n' + page.trim();
		if( page.startsWith("\n{") && page.endsWith("}")){
			page = "[[start]]\n<pre>\n" + page + "\n</pre>\n";
		}

		pres = page.match( /\n<pre>[^]*?\n<\/pre>/g );
		if( !pres) {
			pres = [];
		}	
		comments = page.match( /<!--[^]*?-->/g );
		if( !comments) {
			comments = [];
		}	
		for( i = 0; i < pres.length; i++) {
			page = page.replace( pres[0], "<PRESECTION>");
		}
		for( i = 0; i < comments.length; i++) {
			page = page.replace( comments[0], "<COMMENTSECTION>");
		}
		page = applyTransforms( page);
		for( i = 0; i < pres.length; i++) {
			page = page.replace(  "<PRESECTION>", pres[0]);
		}
		for( i = 0; i < comments.length; i++) {
			page = page.replace( "<COMMENTSECTION>", comments[0]);
		}
		return addHeaders( page);
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function unlockWikiPage( ulsuccess, ulfail) {
	try {
		var req = new XMLHttpRequest();
		var d = {
			Page: document.getElementById('PageName').innerText,
			Lock: myLockString
		};
		req.onload = function() {
			setTimeout( function(){ ulsuccess( d.Page, req);}, 10);
		};
		req.timeout = 3000;
		req.ontimeout =  function() {
			setTimeout( function(){ ulfail( d.Page, req);}, 10);
		};
		req.open('POST', '/jsUser/Wikiunlock', true);
		req.send( JSON.stringify( d));
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function updateImages() {
	try {
		debugLog( 'updateImages');
		var imageIdCount = 0, d = currentImageIndex[ currentImageIndex.length - 1].dir;
		var i, j, len = d.length;
		var e = document.getElementById('ShowImagesArea');
		var s = '<table><tr><td style="width:25%"></td><td style="width:25%"></td><td style="width:25%"></td><td style="width:25%"></td></tr>';
		for( i = 0; i < len;) {
			s += '<tr>';
			var t = '<tr>'
			for( j = 0; j < 4 && i < len; i++) {
				if( d[i].file) {
					var name = d[i].file;
					var cn = name.toUpperCase();
					if( name.indexOf(".DS_Store") < 0) {
						if(cn.endsWith( ".JPG") || cn.endsWith( ".JEPG")) {
							var id = 'imageID' + imageIdCount++ + 'IMG';
							var tt = '[[|'+ name + '|' + name + '|25|transform:rotate(0deg) scale(0.5);]]';
							s += '<td><img class="imageThumbnail" id="' + id + '" onclick="imageClick( this, event, \''+ id+ '\',\'' + tt + '\')" width="80%" src="media/' + name + '">';
							s += '</td>';
							t += '<td>' + name + '</td>';
							j++;
						} else if( cn.endsWith("MOV") || cn.endsWith("MKV") || cn.endsWith("MP4") || cn.endsWith("M4V")) {
							var id = 'imageID' + imageIdCount++ + 'IMG';
							var tt = '[[>'+ name +'|50]]';
							s += '<td><video controls muted class="videoThumbnail" id="' + id + '" onclick="imageClick( this, event, \''+ id+ '\',\'' + tt + '\')" width="80%"><source src="media/' + name + '"></video>';
							s += '</td>';
							t += '<td>' + name + '</td>';
							j++;
						}
					}
				} else if(d[i].dir) {
					var dir = d[i].dir.path;
					s += '<td><button onclick="onDirClick(' + i +' )">' + dir + '</button></td>';
					t += '<td></td>';
					j++;
				} else {

				}
			}
			s += '</tr>' + t + '</tr>';
		}
		s += '</table>';
		e.innerHTML = s;
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function wikiPageLoaded( page, version, req) {
	try {
		if( req.status == 200 ) {
			document.getElementById('PageName').innerText = page;
			var d = parsePage(cleanResponse(req.responseText));
			document.getElementById('ShowArea').innerText = d[0];
			document.getElementById('EditArea').value = d[1];
			document.getElementById('DisplayArea').innerHTML = transform( d[1]);
			document.getElementById( 'ShowButton').disabled = false;
			document.getElementById( 'EditButton').disabled = false;
			debugLog( 'wikiPageLoaded: ' + page + ':' + version + '::' + req.status);
		} else { 
			wikiPageLoadFailed( page,version, req);
		}
	} catch( e) {
		javascriptExceptionLog( e);
	}
}
function wikiPageLoadFailed( page, version, req) {
	try {
		errorLog( 'wikiPageLoadFailed: ' + currentPage + '::' + req.status);
	} catch( e) {
		javascriptExceptionLog( e);
	}
}

