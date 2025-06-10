<?php

if(!isset($_POST['s'])) {
  header("Location: /index.html");
  die();
}

$username = $_POST['username'];
$password = $_POST['password'];

$db = new SQLite3('login.db', SQLITE3_OPEN_READONLY);

$q = $db->querySingle("SELECT COUNT(*) FROM \"users\" WHERE \"username\"='{$username}' AND \"password\"='{$password}'");
if($q > 1) {
  die("CTF{SQLi_st1ll_v4l1d!}");
} else {
  header("Location: /index.html");
  die();
}
