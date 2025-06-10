#!/usr/bin/perl
use strict;
use warnings;
use IO::Socket::INET;

my $name=$ENV{TASK};

my $s = IO::Socket::INET->new(Listen => 1, Backlog => 1, Proto => "tcp", LocalPort => "3001${name}");
die("socket error") unless $s;

print "Listening :::3001${name}\n";

while(1) {
	my $c = $s->accept();
	next unless $c;
	print "New client connected...";
	if(open(my $fp, "~/shell${name}/run_shell.sh|")) {
		my $url = <$fp>;
		chomp $url;
		close $fp;
		print "givin $url\n";
		print $c <<EOF
HTTP/1.1 302 Found
Connection: close
Location: $url

.
EOF
		;
	} else {
		print $c <<EOF
HTTP/1.1 503 Error
Connection: close
Content-Type: text/plain

Internal spawn error, try again later
EOF
		;
		print "giving error\n";
	}
	$c->close();
}
