<html>
<head>
<title>My Blog</title>
</head>
<body>
<h1>Welcome to my <a href="/">blog</a></h1>


<?php
$page = 'index.html';
if(isset($_GET['page'])) {
  $page = $_GET['page'];
}
$data = file_get_contents("page/{$page}");
echo $data;
?>
</body>
</html>
